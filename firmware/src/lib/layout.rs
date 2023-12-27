use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::Error;
use core::result::Result;
use heapless::Vec;

#[derive(Debug, Serialize, Deserialize)]
pub struct Keymap {
  layers: Vec<Vec<Behavior, 128>, 8>,
}

macro_rules! make_behavior_enum {
  ( $(($variant:ident, $label:literal)),* $(,)? ) => {
    #[derive(Debug)]
    pub enum Behavior {
      $($variant),*
      }
    impl Serialize for Behavior {
      fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
      where S: Serializer
      {
        ser.serialize_str(match *self {
          $(Behavior::$variant => $label),*
        })
      }
    }
    impl <'de> Deserialize<'de> for Behavior {
      fn deserialize<D>(de: D) -> Result<Self, D::Error>
      where D: Deserializer<'de>
      {
        let s: &str = <&str>::deserialize(de)?;
        Ok(match s {
          $($label => Behavior::$variant),* ,
          &_ => {
            return Result::Err(D::Error::custom("invalid behavior"));
          }
        })
      }
    }
  }
}

make_behavior_enum!(
  (Tab, "KC_TAB"),
  (Space, "KC_SPC"),
  (Backspace, "KC_BSPC"),
  (LCtrl, "KC_LCTL"),
  (RCtrl, "KC_RCTL"),
  (LAlt, "KC_LALT"),
  (RAlt, "KC_RALT"),
  (LShift, "KC_LSFT"),
  (RShift, "KC_RSFT"),
  (LGui, "KC_LGUI"),
  (A, "KC_A"),
  (B, "KC_B"),
  (C, "KC_C"),
  (D, "KC_D"),
  (E, "KC_E"),
  (F, "KC_F"),
  (G, "KC_G"),
  (H, "KC_H"),
  (I, "KC_I"),
  (J, "KC_J"),
  (K, "KC_K"),
  (L, "KC_L"),
  (M, "KC_M"),
  (N, "KC_N"),
  (O, "KC_O"),
  (P, "KC_P"),
  (Q, "KC_Q"),
  (R, "KC_R"),
  (S, "KC_S"),
  (T, "KC_T"),
  (U, "KC_U"),
  (V, "KC_V"),
  (W, "KC_W"),
  (X, "KC_X"),
  (Y, "KC_Y"),
  (Z, "KC_Z"),
  (Comma, "KC_COMM"),
  (Dot, "KC_DOT"),
  (Slash, "KC_SLSH"),
  (Quote, "KC_QUOT"),
);

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn can_deser_keymap() -> Result<(), String> {
    let (_layout, _bytes_read): (Keymap, usize) =
      serde_json::from_slice(include_bytes!("../../keymaps/split-42.json"))
      .map_err(|e| format!("{}", e))?;
    Ok(())
  }
}
