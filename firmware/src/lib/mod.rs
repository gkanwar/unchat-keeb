#![cfg_attr(not(test), no_std)]

use core::convert::Infallible;
use ehal::blocking::delay::DelayUs;
use ehal::digital::v2::{InputPin, OutputPin};

pub mod layout;
pub mod bus;
pub mod board;
pub mod switch_matrix;
pub mod led_matrix;
pub mod vkeyboard;
pub mod usb;

pub mod error;
// re-export all error types
pub use crate::error::*;

pub mod prelude {
  pub const BUS_WIDTH: usize = 6;
  
  pub const MAX_KEYS: usize = 128;
  pub const MAX_LAYERS: usize = 8;
  pub const KEY_MASK_LEN: usize = MAX_KEYS / 64;
  pub type KeyMask = [u64; KEY_MASK_LEN];
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

  pub type KeyUsageMask = [u8; 16];

  pub const MAX_EVENTS: usize = 16;

  pub const USB_CLASS_HID: u8 = 3;
  pub const NKRO_MIN_KEY: u8 = 0x02;
  pub const NKRO_MAX_KEY: u8 = 0x81;
  pub const MIN_MODIFIER: u8 = 0xe0;
  pub const MAX_MODIFIER: u8 = 0xe7;
  
  // re-export all error types
  pub use crate::error::*;
}
use prelude::*;


use bus::{TryIntoOutputPin, TryIntoInputPin};
// TODO: need clock info
pub fn tick<D: DelayUs<u32>, P: InputPin<Error=Infallible>, Q: OutputPin<Error=Infallible>>(
  bus: bus::InputBus<P>,
  bus_lock: bus::BusLock,
  switches: &mut switch_matrix::SwitchMatrix<Q>,
  leds: &mut led_matrix::LedMatrix<Q>,
  vkbd: &mut vkeyboard::VKeyboard,
  delay: &mut D)
  -> Result<(bool, bus::InputBus<P>, bus::BusLock), Error>
where
  P: TryIntoOutputPin<Pin=Q>,
  Q: TryIntoInputPin<Pin=P>,
{
  let mut out_bus = bus.into_output_bus(bus_lock);
  leds.tick(&mut out_bus);
  let (in_bus, mut bus_lock) = out_bus.into_input_bus();
  let mut updated = false;
  for i in 0..switches.num_regs() {
    let (key_events, new_bus_lock) =
      switches.subtick(i as RegIndex, &in_bus, bus_lock, delay)?;
    bus_lock = new_bus_lock;
    let now_updated = vkbd.update(key_events)?;
    updated = updated || now_updated;
    if vkbd.reset {
      break;
    }
  }
  return Ok((updated, in_bus, bus_lock));
}
