#![cfg_attr(not(test), no_std)]

use ehal::blocking::delay::DelayUs;
use ehal::digital::v2::{InputPin, OutputPin};
use core::convert::Infallible;

pub mod layout;
pub mod bus;
pub mod board;
pub mod switch_matrix;
pub mod led_matrix;
pub mod vkeyboard;
pub mod usb;

// re-export all error types
pub mod error;
pub use error::*;

// global consts and type aliases
pub mod consts {
  pub const BUS_WIDTH: usize = 6;
  
  pub const MAX_KEYS: usize = 128;
  pub const MAX_LAYERS: usize = 8;
  pub type KeyMask = (u64, u64);
  pub type KeyIndex = u8;
  pub type LayerMask = u8;
  pub type LayerIndex = u8;

  pub const MAX_ROWS: usize = 8;
  pub const MAX_COLS: usize = 32;
  pub const MAX_REGS: usize = 10;
  pub type RegIndex = u8;
  pub type RegBitIndex = u8;
  pub type RegValue = u8;
  pub type PinIndex = u8;

  pub const MAX_EVENTS: usize = 16;
}
use consts::*;


use bus::{TryIntoOutputPin, TryIntoInputPin};
// TODO: need clock info
pub fn tick<D: DelayUs<u32>, P: InputPin<Error=Infallible>, Q: OutputPin<Error=Infallible>>(
  bus: bus::InputBus<P>,
  switches: &mut switch_matrix::SwitchMatrix<Q>,
  leds: &mut led_matrix::LedMatrix<Q>,
  vkbd: &mut vkeyboard::VKeyboard,
  usb: &mut usb::Usb,
  delay: &mut D)
  -> Result<bus::InputBus<P>, Error>
where
  P: TryIntoOutputPin<Pin=Q>,
  Q: TryIntoInputPin<Pin=P>,
{
  let mut out_bus = bus.into_output_bus();
  leds.tick(&mut out_bus);
  let in_bus = out_bus.into_input_bus();
  for i in 0..switches.num_regs() {
    let key_events = switches.subtick(i as RegIndex, &in_bus, delay)?;
    let usb_events = vkbd.update(key_events)?;
    usb.send(usb_events)?;
  }
  return Ok(in_bus);
}
