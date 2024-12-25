#![cfg_attr(not(test), no_std)]

use core::convert::Infallible;
use ehal::blocking::delay::DelayUs;
use ehal::digital::v2::{InputPin, OutputPin};

pub mod layout;
pub mod board;
pub mod switch_matrix;
pub mod vkeyboard;
pub mod usb;
pub mod error;
pub mod basic;

pub mod prelude {
  pub const BUS_WIDTH: usize = 4;
  pub const SEL_WIDTH: usize = 4;
  
  pub const MAX_KEYS: usize = 128;
  pub const MAX_LAYERS: usize = 8;
  pub const KEY_MASK_WIDTH: usize = 32;
  pub const KEY_MASK_LEN: usize = MAX_KEYS / KEY_MASK_WIDTH;
  pub type KeyMask = [u32; KEY_MASK_LEN];
  pub type KeyIndex = u8;
  pub type LayerMask = u8;
  pub type LayerIndex = u8;

  pub const MAX_ROWS: usize = 8;
  pub const MAX_COLS: usize = 32;
  pub const MAX_REGS: usize = 16;
  pub type RegIndex = u8;
  pub type RegBitIndex = u8;
  pub type RegValue = [u16; BUS_WIDTH];
  pub type PinIndex = u8;

  pub const MAX_EVENTS: usize = 16;

  pub const USB_CLASS_HID: u8 = 3;
  pub const NKRO_MIN_KEY: u8 = 0x02;
  pub const NKRO_MAX_KEY: u8 = 0x81;
  pub const MIN_MODIFIER: u8 = 0xe0;
  pub const MAX_MODIFIER: u8 = 0xe7;
  
  pub trait TryIntoOutputPin {
    type Pin;
    fn try_into_output_pin(self) -> Result<Self::Pin, Error>;
  }
  pub trait TryIntoInputPin {
    type Pin;
    fn try_into_input_pin(self) -> Result<Self::Pin, Error>;
  }

  // re-export all basic types
  pub use crate::basic::*;
  // re-export all error types
  pub use crate::error::*;
}
use prelude::*;

pub mod bus {
  use crate::prelude::*;
  pub trait AnalogBus {
    fn read(&mut self) -> RegValue;
  }
}

// TODO: need clock info
pub fn tick<D: DelayUs<u32>, Q: OutputPin<Error=Infallible>, B: bus::AnalogBus>(
  mut bus: B,
  switches: &mut switch_matrix::SwitchMatrix<Q>,
  vkbd: &mut vkeyboard::VKeyboard,
  delay: &mut D,
  write_fmt: impl Fn(core::fmt::Arguments) -> ())
  -> Result<(bool, B), Error>
{
  let mut updated = false;
  for i in 0..switches.num_regs() {
    let key_events = switches.subtick(i as RegIndex, &mut bus, delay, &write_fmt)?;
    let now_updated = vkbd.update(key_events, &write_fmt)?;
    updated = updated || now_updated;
    if vkbd.reset {
      break;
    }
  }
  return Ok((updated, bus));
}
