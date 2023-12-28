use heapless::Vec;
use ehal::digital::v2::{InputPin, OutputPin};
use ehal::blocking::delay::DelayUs;
use core::convert::Infallible;

use crate::board::RegMap;
use crate::vkeyboard::KeyEvent;
use crate::Error;
use crate::consts::*;
use crate::bus::InputBus;

pub struct SwitchMatrix<Q: OutputPin> {
  reg_map: RegMap,
  reg_en_pins: Vec<Q, MAX_REGS>,
  reg_state: Vec<RegValue, MAX_REGS>,
}

impl<Q: OutputPin> SwitchMatrix<Q> {
  pub fn new(reg_map: RegMap, mut reg_en_pins: Vec<Q, MAX_REGS>) -> Result<Self, Error> {
    if reg_map.regs.len() != reg_en_pins.len() {
      return Result::Err(Error::SizeMismatch);
    }
    // disable all switch registers (inverted enable pins)
    for i in 0..reg_en_pins.len() {
      reg_en_pins[i].set_high();
    }
    let mut reg_state = Vec::<RegValue, MAX_REGS>::new();
    reg_state.resize_default(reg_map.regs.len());
    Ok(Self {
      reg_map: reg_map,
      reg_en_pins: reg_en_pins,
      reg_state: reg_state,
    })
  }

  pub fn num_regs(&self) -> usize {
    self.reg_map.regs.len()
  }

  pub fn subtick<D: DelayUs<u32>, P: InputPin<Error=Infallible>>(
    &mut self, i_reg: RegIndex, bus: &InputBus<P>, delay: &mut D)
    -> Result<Vec<KeyEvent, BUS_WIDTH>, Error>
  {
    let old_state = self.reg_state[i_reg as usize];
    let mut events = Vec::<KeyEvent, BUS_WIDTH>::new();

    self.reg_en_pins[i_reg as usize].set_low(); // enable
    delay.delay_us(3);
    let new_state = bus.read();
    self.reg_en_pins[i_reg as usize].set_high(); // disable

    // TODO: software debounce?
    for i in 0..BUS_WIDTH {
      let new_bit = (new_state >> i) & 1;
      let old_bit = (old_state >> i) & 1;
      let key: KeyIndex = self.reg_map.regs[i_reg as usize][i];
      if new_bit != old_bit {
        match new_bit {
          0 => events.push(KeyEvent::Up(key)).map_err(|_| Error::VecOverflow)?,
          _ => events.push(KeyEvent::Down(key)).map_err(|_| Error::VecOverflow)?,
        }
      }
    }

    self.reg_state[i_reg as usize] = new_state;

    return Ok(events);
  }
}
