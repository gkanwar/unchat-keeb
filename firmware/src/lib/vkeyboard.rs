use heapless::Vec;
use core::result::Result;

use crate::consts::*;
use crate::Error;
use crate::usb::UsbEvent;

// Virtual keyboard state follows the QMK model:
// - One default layer that is always active
// - Other layers can be active or inactive depending on modifiers and toggles
// - The layers have priority from highest to lowest
// - A given key triggers the highest active non-transparent function
// Extra history needs to be maintained for key-up events, beacuse the key-down
// event may have been made with a different active layer set:
// - A matrix of the layer on which each key was last activated
// - The time of the key down event to distinguish taps from holds

pub struct VKeyboard {
  default_layer: LayerIndex,
  active_layer_mask: LayerMask,
  key_down_layer: Vec<LayerIndex, MAX_KEYS>,
  key_down_mask: KeyMask,
}

impl VKeyboard {
  pub fn new(n_keys: usize) -> Result<Self, Error> {
    let mut kd_layer = Vec::new();
    kd_layer.resize_default(n_keys).map_err(|_| Error::VecOverflow)?;
    Ok(Self {
      default_layer: 0,
      active_layer_mask: 0,
      key_down_layer: kd_layer,
      key_down_mask: (0, 0),
    })
  }

  pub fn update(
    &mut self,
    key_events: Vec<KeyEvent, BUS_WIDTH>)
    -> Result<Vec<UsbEvent, MAX_EVENTS>, Error>
  {
    todo!("interesting layer logic goes here");
  }
}

pub enum KeyEvent {
  Up(KeyIndex),
  Down(KeyIndex),
}
