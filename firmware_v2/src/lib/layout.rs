use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::Error as DError;
use serde::ser::Error as SError;
use core::result::Result;
use heapless::Vec;
use core::str;
use core::fmt::Write;

use crate::prelude::*;

#[derive(Debug,Serialize,Deserialize)]
pub struct Keymap {
  pub layout: LayoutKind,
  pub layers: Vec<Vec<Behavior, MAX_KEYS>, MAX_LAYERS>,
}

#[derive(Debug,Clone,Copy)]
pub enum LayoutKind {
  LayoutSplit3x6_2,
}

impl Serialize for LayoutKind {
  fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
  where S: Serializer
  {
    ser.serialize_str(match *self {
      LayoutKind::LayoutSplit3x6_2 => "LAYOUT_split_3x6_2"
    })
  }
}

impl<'de> Deserialize<'de> for LayoutKind {
  fn deserialize<D>(de: D) -> Result<Self, D::Error>
  where D: Deserializer<'de>
  {
    let s: &str = <&str>::deserialize(de)?;
    Ok(match s {
      "LAYOUT_split_3x6_2" => LayoutKind::LayoutSplit3x6_2,
      &_ => {
        return Result::Err(D::Error::custom("invalid layout kind"));
      }
    })
  }
}

pub struct Layout {
  pub matrix_pos: Vec<(usize, usize), MAX_KEYS>,
  pub render_pos: Vec<(usize, usize), MAX_KEYS>,
  pub render_rows: usize,
  pub render_cols: usize,
}

pub fn get_layout(kind: LayoutKind) -> Layout {
  match kind {
    LayoutKind::LayoutSplit3x6_2 => {
      let n_keys = 2*(3*6 + 2);
      let matrix_pos = [
        (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (4,0), (4,1), (4,2), (4,3), (4,4), (4,5),
        (1,0), (1,1), (1,2), (1,3), (1,4), (1,5), (5,0), (5,1), (5,2), (5,3), (5,4), (5,5),
        (2,0), (2,1), (2,2), (2,3), (2,4), (2,5), (6,0), (6,1), (6,2), (6,3), (6,4), (6,5),
        (3,0), (3,1), (7,0), (7,1),
      ];
      let render_pos = [
        (0,0), (0,1), (0,2), (0,3), (0,4), (0,5), (0,7), (0,8), (0,9), (0,10), (0,11), (0,12),
        (1,0), (1,1), (1,2), (1,3), (1,4), (1,5), (1,7), (1,8), (1,9), (1,10), (1,11), (1,12),
        (2,0), (2,1), (2,2), (2,3), (2,4), (2,5), (2,7), (2,8), (2,9), (2,10), (2,11), (2,12),
        (3,4), (3,5), (3,7), (3,8),
      ];
      let render_rows = 4;
      let render_cols = 13;
      assert_eq!(matrix_pos.len(), n_keys);
      assert_eq!(render_pos.len(), n_keys);
      Layout {
        matrix_pos: Vec::from_slice(&matrix_pos).unwrap(),
        render_pos: Vec::from_slice(&render_pos).unwrap(),
        render_rows,
        render_cols,
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
  (Delete, "KC_DEL"),
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
  (Caret, "KC_CIRC"),
  (Percent, "KC_PERC"),
  (Exclamation, "KC_EXLM"),
  (LParen, "KC_LPRN"),
  (RParen, "KC_RPRN"),
  (At, "KC_AT"),
  (Ampersand, "KC_AMPR"),
  (Asterisk, "KC_ASTR"),
  (Hash, "KC_HASH"),
  (Tilde, "KC_TILD"),
  (Dollar, "KC_DLR"),
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

pub fn behavior_to_utf8(b: Behavior) -> Vec<u8, 64> {
  let mut buf = WriteBuf::<64>::new();
  use Behavior::*;
  match b {
    LayerGoto(i) => write!(buf, "TO{}", i),
    LayerMod(i) => write!(buf, "MO{}", i),
    LayerToggle(i) => write!(buf, "TG{}", i),
    LayerTapToggle(i) => write!(buf, "TT{}", i),
    Enter => write!(buf, "⮐"),
    Tab => write!(buf, "Tab"),
    Space => write!(buf, "Spc"),
    Backspace => write!(buf, "Bsp"),
    Escape => write!(buf, "Esc"),
    ArrowUp => write!(buf, "↑"),
    ArrowDown => write!(buf, "↓"),
    ArrowLeft => write!(buf, "←"),
    ArrowRight => write!(buf, "→"),
    Home => write!(buf, "Hom"),
    End => write!(buf, "End"),
    PageUp => write!(buf, "PgU"),
    PageDown => write!(buf, "PgD"),
    Delete => write!(buf, "Del"),
    A => write!(buf, "A"),
    B => write!(buf, "B"),
    C => write!(buf, "C"),
    D => write!(buf, "D"),
    E => write!(buf, "E"),
    F => write!(buf, "F"),
    G => write!(buf, "G"),
    H => write!(buf, "H"),
    I => write!(buf, "I"),
    J => write!(buf, "J"),
    K => write!(buf, "K"),
    L => write!(buf, "L"),
    M => write!(buf, "M"),
    N => write!(buf, "N"),
    O => write!(buf, "O"),
    P => write!(buf, "P"),
    Q => write!(buf, "Q"),
    R => write!(buf, "R"),
    S => write!(buf, "S"),
    T => write!(buf, "T"),
    U => write!(buf, "U"),
    V => write!(buf, "V"),
    W => write!(buf, "W"),
    X => write!(buf, "X"),
    Y => write!(buf, "Y"),
    Z => write!(buf, "Z"),
    Num0 => write!(buf, "0"),
    Num1 => write!(buf, "1"),
    Num2 => write!(buf, "2"),
    Num3 => write!(buf, "3"),
    Num4 => write!(buf, "4"),
    Num5 => write!(buf, "5"),
    Num6 => write!(buf, "6"),
    Num7 => write!(buf, "7"),
    Num8 => write!(buf, "8"),
    Num9 => write!(buf, "9"),
    Comma => write!(buf, ","),
    Dot => write!(buf, "."),
    Slash => write!(buf, "/"),
    Backslash => write!(buf, "\\"),
    Quote => write!(buf, "'"),
    LBrace => write!(buf, "["),
    RBrace => write!(buf, "]"),
    LCtrl | RCtrl => write!(buf, "Ctl"),
    LAlt | RAlt => write!(buf, "Alt"),
    LShift | RShift => write!(buf, "Sft"),
    LGui | RGui => write!(buf, "Cmd"),
    // TODO: other symbols
    _ => write!(buf, "<?>"),
  }.unwrap();
  return buf.data;
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn can_deser_keymap() -> Result<(), String> {
    let (_layout, _bytes_read): (Keymap, usize) =
      serde_json::from_slice(include_bytes!("../../keymaps/split-40-colemak.json"))
      .map_err(|e| format!("{}", e))?;
    Ok(())
  }
}
