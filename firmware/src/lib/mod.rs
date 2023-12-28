#![cfg_attr(not(test), no_std)]

pub mod layout;
pub mod bus;
// pub mod key_matrix;
// pub mod led_matrix;
// pub mod vkeyboard;
// pub mod usb;

// re-export all error types
pub mod error;
pub use error::*;

// pub fn step(
//   bus: &bus::Bus, keys: &mut key_matrix::KeyMatrix,
//   leds: &mut led_matrix::LedMatrix, vkbd: &mut vkeyboard::VKeyboard,
//   usb: &usb::Usb)
//   -> Result<(), Error>
// {
//   leds.step(&bus)?;
//   let key_events = keys.step(&bus)?;
//   let usb_events = vkbd.update(events)?;
//   usb.send(usb_events)?;
// }
