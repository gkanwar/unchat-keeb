use heapless::Vec;
use ehal::digital::v2::{InputPin, OutputPin};
use ehal::blocking::delay::DelayUs;
use core::convert::Infallible;

use crate::bus::AnalogBus;
use crate::board::{RegMap, Polarity, SwitchSettings};
use crate::vkeyboard::KeyEvent;
use crate::prelude::*;

#[derive(Debug)]
struct SwitchState {
  is_down: bool,
  settings: SwitchSettings,
}

impl SwitchState {
  fn new(settings: SwitchSettings) -> Self {
    Self {
      is_down: false,
      settings,
    }
  }
}

#[derive(Debug, Copy, Clone)]
enum RegEvent {
  None, SwitchDown, SwitchUp,
}

struct RegState {
  is_enabled: [bool; BUS_WIDTH],
  state: [SwitchState; BUS_WIDTH],
}

const ADC_MAX: u16 = 4096;

impl RegState {
  fn new(calib: [SwitchSettings; BUS_WIDTH]) -> Self {
    Self {
      is_enabled: [false; BUS_WIDTH],
      state: calib.into_iter().map(SwitchState::new)
        .collect::<Vec<SwitchState, BUS_WIDTH>>().into_array().unwrap()
    }
  }
  fn update(&mut self, new_values: RegValue) -> [RegEvent; BUS_WIDTH] {
    let mut events = [RegEvent::None; BUS_WIDTH];
    for i in 0..BUS_WIDTH {
      if !self.is_enabled[i] {
        continue;
      }
      // polarity determines the direction of the triggers
      let state = &mut self.state[i];
      let settings = state.settings;
      let value_norm = new_values[i] as f32 / ADC_MAX as f32;
      // FORNOW:
      // if state.is_down {
      //   state.is_down = false;
      //   events[i] = RegEvent::SwitchUp;
      // }
      // else {
      //   state.is_down = true;
      //   events[i] = RegEvent::SwitchDown;
      // }
      match settings.polarity {
        Polarity::S => {
          // we use hysteresis to debounce
          assert!(settings.trig_up > settings.trig_down);
          if state.is_down && value_norm > settings.trig_up {
            state.is_down = false;
            events[i] = RegEvent::SwitchUp;
          }
          else if !state.is_down && value_norm < settings.trig_down {
            state.is_down = true;
            events[i] = RegEvent::SwitchDown;
          }
        }
        Polarity::N => {
          // we use hysteresis to debounce
          assert!(settings.trig_up < settings.trig_down);
          if state.is_down && value_norm < settings.trig_up {
            state.is_down = false;
            events[i] = RegEvent::SwitchUp;
          }
          else if !state.is_down && value_norm > settings.trig_down {
            state.is_down = true;
            events[i] = RegEvent::SwitchDown;
          }
        }
      }
    }
    return events;
  }    
}

pub struct SwitchMatrix<Q: OutputPin> {
  reg_map: RegMap,
  sel_pins: [Q; SEL_WIDTH],
  reg_state: Vec<RegState, MAX_REGS>,
}

impl<Q: OutputPin> SwitchMatrix<Q> {
  pub fn new(reg_map: RegMap, sel_pins: [Q; SEL_WIDTH]) -> Result<Self, Error> {
    let mut reg_state: Vec<RegState, MAX_REGS> =
      reg_map.calibration.clone().into_iter().map(RegState::new).collect();
    for i in 0..reg_map.regs.len() {
      for j in 0..BUS_WIDTH {
        if reg_map.regs[i][j].is_some() {
          reg_state[i].is_enabled[j] = true;
        }
      }
    }
    Ok(Self {reg_map, sel_pins: sel_pins.into(), reg_state})
  }

  pub fn num_regs(&self) -> usize {
    self.reg_map.regs.len()
  }

  pub fn subtick<D: DelayUs<u32>, B: AnalogBus>(
    &mut self, i_reg: RegIndex, bus: &mut B, delay: &mut D,
    write_fmt: impl Fn(core::fmt::Arguments) -> ())
    -> Result<Vec<KeyEvent, BUS_WIDTH>, Error>
  {
    let reg_state = &mut self.reg_state[i_reg as usize];
    let mut events = Vec::<KeyEvent, BUS_WIDTH>::new();

    // guarded by bus lock
    for i in 0..SEL_WIDTH {
      if (i_reg >> i) % 2 == 0 {
        self.sel_pins[i].set_low().map_err(|_| Error::PinConfigError)?;
      }
      else {
        self.sel_pins[i].set_high().map_err(|_| Error::PinConfigError)?;
      }
    }
    delay.delay_us(2);
    // TODO: multisampling to reduce noise
    let read_values = bus.read();
    // write_fmt(format_args!("Read values {}: {:?}\r\n", i_reg, read_values));
    let reg_events = reg_state.update(read_values);
    delay.delay_us(1);

    for i in 0..BUS_WIDTH {
      let key: KeyIndex = match self.reg_map.regs[i_reg as usize][i] {
        Some(key) => key,
        None => continue,
      };
      match reg_events[i] {
        RegEvent::None => {},
        RegEvent::SwitchUp => events.push(KeyEvent::Up(key)).map_err(|_| Error::VecOverflow)?,
        RegEvent::SwitchDown => events.push(KeyEvent::Down(key)).map_err(|_| Error::VecOverflow)?,
      }
    }

    return Ok(events);
  }
}
