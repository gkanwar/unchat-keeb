use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::Error as DError;
use serde::ser::Error as SError;
use core::result::Result;
use heapless::Vec;
use core::str;

use crate::prelude::*;

#[derive(Debug,Serialize,Deserialize)]
pub struct Keymap {
  pub layout: LayoutKind,
  pub layers: Vec<Vec<Behavior, MAX_KEYS>, MAX_LAYERS>,
}

#[derive(Debug,Clone,Copy)]
pub enum LayoutKind {
  LayoutSplit3x6_3,
}

impl Serialize for LayoutKind {
  fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    ser.serialize_str(match *self {
      LayoutKind::LayoutSplit3x6_3 => "LAYOUT_split_3x6_3"
    })
  }
}

impl<'de> Deserialize<'de> for LayoutKind {
  fn deserialize<D>(de: D) -> Result<Self, D::Error>
  where D: Deserializer<'de>
  {
    let s: &str = <&str>::deserialize(de)?;
    Ok(match s {
      "LAYOUT_split_3x6_3" => LayoutKind::LayoutSplit3x6_3,
      &_ => {
        return Result::Err(D::Error::custom("invalid layout kind"));
      }
    })
  }
}

pub struct Layout {
  pub positions: Vec<(usize, usize), MAX_KEYS>,
}

pub fn get_layout(kind: LayoutKind) -> Layout {
  match kind {
    LayoutKind::LayoutSplit3x6_3 => {
      let n_keys = 2*(3*6 + 3);
      let pos = [
        (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (4,0), (4,1), (4,2), (4,3), (4,4), (4,5),
        (1,0), (1,1), (1,2), (1,3), (1,4), (1,5), (5,0), (5,1), (5,2), (5,3), (5,4), (5,5),
        (2,0), (2,1), (2,2), (2,3), (2,4), (2,5), (6,0), (6,1), (6,2), (6,3), (6,4), (6,5),
        (3,0), (3,1), (3,2), (7,0), (7,1), (7,2),
      ];
      assert_eq!(pos.len(), n_keys);
      Layout {
        positions: Vec::from_slice(&pos).unwrap()
      }
    }
  }
}

fn make_layer_str(
  prefix: &str, layer: LayerIndex, layer_buf: &mut [u8])
  -> Result<(), &'static str>
{
  if prefix.len() != 2 {
    return Result::Err("invalid layer prefix");
  }
  layer_buf[0] = prefix.bytes().nth(0).ok_or("invalid prefix")?;
  layer_buf[1] = prefix.bytes().nth(1).ok_or("invalid prefix")?;
  // NOTE: panics on encoding problem
  '('.encode_utf8(&mut layer_buf[2..3]);
  char::from_digit(layer as u32, 10).ok_or("invalid layer")?
    .encode_utf8(&mut layer_buf[3..4]);
  ')'.encode_utf8(&mut layer_buf[4..5]);
  return Ok(());
}

macro_rules! make_behavior_enum {
  ( $(($variant:ident, $label:literal)),* $(,)? ) => {
    #[derive(Debug,Copy,Clone)]
    pub enum Behavior {
      $($variant),* ,
      LayerGoto(LayerIndex),
      LayerMod(LayerIndex),
      LayerToggle(LayerIndex),
      LayerTapToggle(LayerIndex),
    }
    impl Serialize for Behavior {
      fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
      where S: Serializer
      {
        let mut layer_buf: [u8; 5] = [0; 5];
        ser.serialize_str(match *self {
          $(Behavior::$variant => $label),* ,
          Behavior::LayerGoto(i) => {
            make_layer_str("TO", i, &mut layer_buf).map_err(S::Error::custom)?;
            str::from_utf8(&layer_buf[..]).map_err(S::Error::custom)?
          }
          Behavior::LayerMod(i) => {
            make_layer_str("MO", i, &mut layer_buf).map_err(S::Error::custom)?;
            str::from_utf8(&layer_buf[..]).map_err(S::Error::custom)?
          },
          Behavior::LayerToggle(i) => {
            make_layer_str("TG", i, &mut layer_buf).map_err(S::Error::custom)?;
            str::from_utf8(&layer_buf[..]).map_err(S::Error::custom)?
          },
          Behavior::LayerTapToggle(i) => {
            make_layer_str("TT", i, &mut layer_buf).map_err(S::Error::custom)?;
            str::from_utf8(&layer_buf[..]).map_err(S::Error::custom)?
          },
        })
      }
    }
    impl <'de> Deserialize<'de> for Behavior {
      fn deserialize<D>(de: D) -> Result<Self, D::Error>
      where D: Deserializer<'de>
      {
        let s: &str = <&str>::deserialize(de)?;
        if s.starts_with("TO(") && s.ends_with(")") {
          let i: LayerIndex = s[3..s.len()-1].parse::<LayerIndex>()
            .map_err(D::Error::custom)?;
          return Ok(Behavior::LayerGoto(i));
        }
        if s.starts_with("MO(") && s.ends_with(")") {
          let i: LayerIndex = s[3..s.len()-1].parse::<LayerIndex>()
            .map_err(D::Error::custom)?;
          return Ok(Behavior::LayerMod(i));
        }
        if s.starts_with("TG(") && s.ends_with(")") {
          let i: LayerIndex = s[3..s.len()-1].parse::<LayerIndex>()
            .map_err(D::Error::custom)?;
          return Ok(Behavior::LayerToggle(i));
        }
        if s.starts_with("TT(") && s.ends_with(")") {
          let i: LayerIndex = s[3..s.len()-1].parse::<LayerIndex>()
            .map_err(D::Error::custom)?;
          return Ok(Behavior::LayerTapToggle(i));
        }
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
  // special keys
  (Enter, "KC_ENT"),
  (Tab, "KC_TAB"),
  (Space, "KC_SPC"),
  (Backspace, "KC_BSPC"),
  (Escape, "KC_ESC"),
  (ArrowUp, "KC_UP"),
  (ArrowDown, "KC_DOWN"),
  (ArrowLeft, "KC_LEFT"),
  (ArrowRight, "KC_RGHT"),
  (Home, "KC_HOME"),
  (End, "KC_END"),
  (PageUp, "KC_PGUP"),
  (PageDown, "KC_PGDN"),
  // letters
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
  // numbers
  (Num0, "KC_0"),
  (Num1, "KC_1"),
  (Num2, "KC_2"),
  (Num3, "KC_3"),
  (Num4, "KC_4"),
  (Num5, "KC_5"),
  (Num6, "KC_6"),
  (Num7, "KC_7"),
  (Num8, "KC_8"),
  (Num9, "KC_9"),
  // f-keys
  (F1, "KC_F1"),
  (F2, "KC_F2"),
  (F3, "KC_F3"),
  (F4, "KC_F4"),
  (F5, "KC_F5"),
  (F6, "KC_F6"),
  (F7, "KC_F7"),
  (F8, "KC_F8"),
  (F9, "KC_F9"),
  (F10, "KC_F10"),
  (F11, "KC_F11"),
  (F12, "KC_F12"),
  // symbols
  (Comma, "KC_COMM"),
  (Dot, "KC_DOT"),
  (Slash, "KC_SLSH"),
  (Backslash, "KC_BSLS"),
  (Quote, "KC_QUOT"),
  (LBrace, "KC_LBRC"),
  (RBrace, "KC_RBRC"),
  (Grave, "KC_GRV"),
  (Semicolon, "KC_SCLN"),
  (Equals, "KC_EQL"),
  (Minus, "KC_MINS"),
  // special functions
  (PrintScreen, "KC_PSCR"),
  (VolMute, "KC_MUTE"),
  (VolUp, "KC_VOLU"),
  (VolDown, "KC_VOLD"),
  // see to layer below
  (Transparent, "KC_TRNS"),
  // noop
  (Noop, "KC_NO"),
  // modifiers
  (LCtrl, "KC_LCTL"),
  (RCtrl, "KC_RCTL"),
  (LAlt, "KC_LALT"),
  (RAlt, "KC_RALT"),
  (LShift, "KC_LSFT"),
  (RShift, "KC_RSFT"),
  (LGui, "KC_LGUI"),
  (RGui, "KC_RGUI"),
  // keyboard controls
  (BacklightToggle, "BL_TOGG"),
  (BacklightUp, "BL_UP"),
  (BacklightDown, "BL_DOWN"),
  (Reset, "QK_BOOT"),
);

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn can_deser_keymap() -> Result<(), String> {
    let (_layout, _bytes_read): (Keymap, usize) =
      serde_json::from_slice(include_bytes!("../../keymaps/split-42-colemak.json"))
      .map_err(|e| format!("{}", e))?;
    Ok(())
  }
}
