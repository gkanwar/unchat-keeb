use ehal::digital::v2::OutputPin;
use ehal::digital::v2::InputPin;
use ehal::digital::v2::PinState;
use crate::Error;
use core::convert::Infallible;

use crate::consts::*;

pub trait TryIntoOutputPin {
  type Pin;
  fn try_into_output_pin(self) -> Result<Self::Pin, Error>;
}
pub trait TryIntoInputPin {
  type Pin;
  fn try_into_input_pin(self) -> Result<Self::Pin, Error>;
}

pub struct InputBus<P: InputPin>
{
  pub pins: [P; BUS_WIDTH]
}

impl<P: InputPin<Error=Infallible>> InputBus<P> {
  pub fn read(&self) -> RegValue {
    let mut value: RegValue = 0;
    for i in 0..BUS_WIDTH {
      value <<= 1;
      if self.pins[i].is_high().unwrap() {
        value |= 1;
      }
    }
    return value;
  }
}

pub struct OutputBus<Q: OutputPin>
{
  pub pins: [Q; BUS_WIDTH]
}

impl<Q: OutputPin<Error=Infallible>> OutputBus<Q> {
  pub fn write(&mut self, state: RegValue) -> () {
    self.pins.iter_mut().enumerate().for_each(
      |i_pin| {
        let (i, pin) = i_pin;
        pin.set_state(match (state >> i) & 1 {
          0 => PinState::Low,
          _ => PinState::High,
        }).unwrap();
      });
  }
}

impl<P: InputPin> InputBus<P>
{
  pub fn into_output_bus<Q: OutputPin>(self) -> OutputBus<Q>
  where P: TryIntoOutputPin<Pin=Q>, Q: TryIntoInputPin<Pin=P>
  {
    OutputBus::<Q> {
      pins: self.pins.map(|p| p.try_into_output_pin().unwrap())
    }
  }
}

impl<P: OutputPin> OutputBus<P>
{
  pub fn into_input_bus<Q: InputPin>(self) -> InputBus<Q>
  where P: TryIntoInputPin<Pin=Q>, Q: TryIntoOutputPin<Pin=P>
  {
    InputBus::<Q> {
      pins: self.pins.map(|p| p.try_into_input_pin().unwrap())
    }
  }
}
