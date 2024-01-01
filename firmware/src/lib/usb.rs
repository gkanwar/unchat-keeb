use heapless::Vec;

use crate::Error;
use crate::prelude::*;

pub struct Usb {

}

impl Usb {
  pub fn send(&mut self, events: Vec<UsbEvent, MAX_EVENTS>) -> Result<(), Error> {
    todo!("usb protocol");
  }
}

pub struct UsbEvent {
  
}
