use ehal::digital::v2::OutputPin;
use ehal::digital::v2::InputPin;
use ehal::digital::v2::PinState;
use crate::Error;
use core::convert::Infallible;

pub trait TryIntoOutputPin {
  type Pin;
  fn try_into_output_pin(self) -> Result<Self::Pin, Error>;
}
pub trait TryIntoInputPin {
  type Pin;
  fn try_into_input_pin(self) -> Result<Self::Pin, Error>;
}

const BUS_WIDTH: usize = 6;

pub struct InputBus<P: InputPin>
{
  pub pins: [P; BUS_WIDTH]
}

pub struct OutputBus<Q: OutputPin>
{
  pub pins: [Q; BUS_WIDTH]
}

impl<Q: OutputPin<Error=Infallible>> OutputBus<Q> {
  pub fn set_state(&mut self, state: [PinState; BUS_WIDTH]) -> () {
    self.pins.iter_mut().enumerate().for_each(
      |i_pin| i_pin.1.set_state(state[i_pin.0]).unwrap());
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
