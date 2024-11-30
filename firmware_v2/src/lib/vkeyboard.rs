use heapless::Vec;
use core::result::Result;

use crate::prelude::*;
use crate::usb::{KeyUsageAndIndex, NKROBootKeyboardReport, KeyboardUsage};
use crate::layout::{Behavior, Keymap};

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
  // virtual state
  default_layer: LayerIndex,
  active_layer_mask: LayerMask,
  key_down_layer: [LayerIndex; MAX_KEYS],
  key_down_mask: KeyMask,
  // virtual keymap
  keymap: Keymap,
  // logical state
  usb_report: NKROBootKeyboardReport,
  pub reset: bool,
}

enum VirtualFunction {
  VBacklightToggle,
  VBacklightUp,
  VBacklightDown,
  VReset,
  VLayerGoto(LayerIndex),
  VLayerMod(LayerIndex),
  VLayerToggle(LayerIndex),
  VLayerTapToggle(LayerIndex),
}

enum Action {
  // send usage to host
  SendKey(KeyUsageAndIndex),
  // internal function
  Internal(VirtualFunction),
  // do nothing
  Nothing,
}
fn behavior_to_action(behavior: Behavior) -> Action {
  use Behavior::*;
  use KeyboardUsage::*;
  use Action::*;
  use VirtualFunction::*;
  type Kui = KeyUsageAndIndex;
  match behavior {
    // SendKey actions
    A => SendKey(Kui::new(KeyboardAa)), // 0x04
    B => SendKey(Kui::new(KeyboardBb)),
    C => SendKey(Kui::new(KeyboardCc)),
    D => SendKey(Kui::new(KeyboardDd)),
    E => SendKey(Kui::new(KeyboardEe)),
    F => SendKey(Kui::new(KeyboardFf)),
    G => SendKey(Kui::new(KeyboardGg)),
    H => SendKey(Kui::new(KeyboardHh)),
    I => SendKey(Kui::new(KeyboardIi)),
    J => SendKey(Kui::new(KeyboardJj)),
    K => SendKey(Kui::new(KeyboardKk)),
    L => SendKey(Kui::new(KeyboardLl)),
    M => SendKey(Kui::new(KeyboardMm)),
    N => SendKey(Kui::new(KeyboardNn)),
    O => SendKey(Kui::new(KeyboardOo)),
    P => SendKey(Kui::new(KeyboardPp)),
    Q => SendKey(Kui::new(KeyboardQq)),
    R => SendKey(Kui::new(KeyboardRr)),
    S => SendKey(Kui::new(KeyboardSs)),
    T => SendKey(Kui::new(KeyboardTt)),
    U => SendKey(Kui::new(KeyboardUu)),
    V => SendKey(Kui::new(KeyboardVv)),
    W => SendKey(Kui::new(KeyboardWw)),
    X => SendKey(Kui::new(KeyboardXx)),
    Y => SendKey(Kui::new(KeyboardYy)),
    Z => SendKey(Kui::new(KeyboardZz)),
    Num1 => SendKey(Kui::new(Keyboard1Exclamation)),
    Num2 => SendKey(Kui::new(Keyboard2At)),
    Num3 => SendKey(Kui::new(Keyboard3Hash)),
    Num4 => SendKey(Kui::new(Keyboard4Dollar)),
    Num5 => SendKey(Kui::new(Keyboard5Percent)),
    Num6 => SendKey(Kui::new(Keyboard6Caret)),
    Num7 => SendKey(Kui::new(Keyboard7Ampersand)),
    Num8 => SendKey(Kui::new(Keyboard8Asterisk)),
    Num9 => SendKey(Kui::new(Keyboard9OpenParens)),
    Num0 => SendKey(Kui::new(Keyboard0CloseParens)),
    Enter => SendKey(Kui::new(KeyboardEnter)),
    Escape => SendKey(Kui::new(KeyboardEscape)),
    Backspace => SendKey(Kui::new(KeyboardBackspace)),
    Tab => SendKey(Kui::new(KeyboardTab)),
    Space => SendKey(Kui::new(KeyboardSpacebar)),
    Minus => SendKey(Kui::new(KeyboardDashUnderscore)),
    Equals => SendKey(Kui::new(KeyboardEqualPlus)),
    LBrace => SendKey(Kui::new(KeyboardOpenBracketBrace)),
    RBrace => SendKey(Kui::new(KeyboardCloseBracketBrace)),
    Backslash => SendKey(Kui::new(KeyboardBackslashBar)),
    /* _ => SendKey(Kui::new(KeyboardNonUSHash)), */
    Semicolon => SendKey(Kui::new(KeyboardSemiColon)),
    Quote => SendKey(Kui::new(KeyboardSingleDoubleQuote)),
    Grave => SendKey(Kui::new(KeyboardBacktickTilde)),
    Comma => SendKey(Kui::new(KeyboardCommaLess)),
    Dot => SendKey(Kui::new(KeyboardPeriodGreater)),
    Slash => SendKey(Kui::new(KeyboardSlashQuestion)),
    /* _ => SendKey(Kui::new(KeyboardCapsLock)), */
    F1 => SendKey(Kui::new(KeyboardF1)),
    F2 => SendKey(Kui::new(KeyboardF2)),
    F3 => SendKey(Kui::new(KeyboardF3)),
    F4 => SendKey(Kui::new(KeyboardF4)),
    F5 => SendKey(Kui::new(KeyboardF5)),
    F6 => SendKey(Kui::new(KeyboardF6)),
    F7 => SendKey(Kui::new(KeyboardF7)),
    F8 => SendKey(Kui::new(KeyboardF8)),
    F9 => SendKey(Kui::new(KeyboardF9)),
    F10 => SendKey(Kui::new(KeyboardF10)),
    F11 => SendKey(Kui::new(KeyboardF11)),
    F12 => SendKey(Kui::new(KeyboardF12)),
    PrintScreen => SendKey(Kui::new(KeyboardPrintScreen)),
    /* _ => SendKey(Kui::new(KeyboardScrollLock)), */
    /* _ => SendKey(Kui::new(KeyboardPause)), */
    /* _ => SendKey(Kui::new(KeyboardInsert)), */
    Home => SendKey(Kui::new(KeyboardHome)),
    PageUp => SendKey(Kui::new(KeyboardPageUp)),
    Delete => SendKey(Kui::new(KeyboardDelete)),
    End => SendKey(Kui::new(KeyboardEnd)),
    PageDown => SendKey(Kui::new(KeyboardPageDown)),
    ArrowRight => SendKey(Kui::new(KeyboardRightArrow)),
    ArrowLeft => SendKey(Kui::new(KeyboardLeftArrow)),
    ArrowDown => SendKey(Kui::new(KeyboardDownArrow)),
    ArrowUp => SendKey(Kui::new(KeyboardUpArrow)), // 0x52
    /* SKIP many keypad functions */
    VolMute => SendKey(Kui::new(KeyboardMute)), // 0x7f
    VolUp => SendKey(Kui::new(KeyboardVolumeUp)),
    VolDown => SendKey(Kui::new(KeyboardVolumeDown)), // 0x81
    /* SKIP many keypad functions */
    LCtrl => SendKey(Kui::new(KeyboardLeftControl)), // 0xe0
    LShift => SendKey(Kui::new(KeyboardLeftShift)),
    LAlt => SendKey(Kui::new(KeyboardLeftAlt)),
    LGui => SendKey(Kui::new(KeyboardLeftGUI)),
    RCtrl => SendKey(Kui::new(KeyboardRightControl)),
    RShift => SendKey(Kui::new(KeyboardRightShift)),
    RAlt => SendKey(Kui::new(KeyboardRightAlt)),
    RGui => SendKey(Kui::new(KeyboardRightGUI)), // 0xe7

    // internal actions
    BacklightToggle => Internal(VBacklightToggle),
    BacklightUp => Internal(VBacklightUp),
    BacklightDown => Internal(VBacklightDown),
    Reset => Internal(VReset),
    LayerGoto(i) => Internal(VLayerGoto(i)),
    LayerMod(i) => Internal(VLayerMod(i)),
    LayerToggle(i) => Internal(VLayerToggle(i)),
    LayerTapToggle(i) => Internal(VLayerTapToggle(i)),

    // anything else
    Transparent => Nothing,
    Noop => Nothing,
  }
}

impl VKeyboard {
  pub fn new(keymap: Keymap) -> Result<Self, Error> {
    Ok(Self {
      default_layer: 0,
      active_layer_mask: 0,
      key_down_layer: [0; MAX_KEYS],
      key_down_mask: [0; KEY_MASK_LEN],
      keymap: keymap,
      usb_report: NKROBootKeyboardReport::default(),
      reset: false,
    })
  }

  fn apply_kui_down(&mut self, kui: KeyUsageAndIndex) {
    let report = &mut self.usb_report;
    match kui {
      KeyUsageAndIndex::Normal { usage, byte, bit } => {
        for i in 0..report.boot_keys.len() {
          if report.boot_keys[i] == 0 {
            report.boot_keys[i] = usage;
            break;
          }
        }
        assert!(bit < 8 && byte < report.nkro_keys.len());
        report.nkro_keys[byte] |= (1 << bit) as u8;
      }
      KeyUsageAndIndex::Modifier { bit } => {
        report.modifier |= (1 << bit) as u8;
      }
    }
  }

  fn apply_kui_up(&mut self, kui: KeyUsageAndIndex) {
    let report = &mut self.usb_report;
    match kui {
      KeyUsageAndIndex::Normal { usage, byte, bit } => {
        for i in (0..report.boot_keys.len()).rev() {
          if report.boot_keys[i] == usage {
            report.boot_keys[i] = 0;
            break;
          }
        }
        assert!(bit < 8 && byte < report.nkro_keys.len());
        report.nkro_keys[byte] &= !((1 << bit) as u8);
      }
      KeyUsageAndIndex::Modifier { bit } => {
        report.modifier &= !((1 << bit) as u8);
      }
    }
  }

  fn apply_vfunc_down(&mut self, vfunc: VirtualFunction) {
    use VirtualFunction::*;
    match vfunc {
      VBacklightToggle => {}, // TODO
      VBacklightUp => {}, // TODO
      VBacklightDown => {}, // TODO
      VReset => {
        self.reset = true;
      },
      VLayerGoto(i) => {
        self.active_layer_mask = 1 << (i as LayerMask);
      },
      VLayerMod(i) => {
        self.active_layer_mask |= 1 << (i as LayerMask);
      },
      VLayerToggle(i) => {
        self.active_layer_mask ^= 1 << (i as LayerMask);
      },
      VLayerTapToggle(i) => {
        self.active_layer_mask |= 1 << (i as LayerMask);
        // TODO: count taps
      },
    }
  }

  fn apply_vfunc_up(&mut self, vfunc: VirtualFunction) {
    use VirtualFunction::*;
    match vfunc {
      VBacklightToggle => {}, // TODO
      VBacklightUp => {}, // TODO
      VBacklightDown => {}, // TODO
      VReset => {},
      VLayerGoto(i) => {},
      VLayerMod(i) => {
        self.active_layer_mask &= !(1 << (i as LayerMask));
      },
      VLayerToggle(i) => {},
      VLayerTapToggle(i) => {
        // TODO: don't disable depending on tap count
        self.active_layer_mask &= !(1 << (i as LayerMask));
      },
    }
  }

  fn key_down(&mut self, idx: KeyIndex) -> Result<bool, Error> {
    for i in (0..self.keymap.layers.len()).rev() {
      if i != self.default_layer as usize && (self.active_layer_mask >> i) & 1 == 0 {
        continue;
      }
      if idx as usize >= self.keymap.layers[i].len() {
        continue;
      }
      let behavior = self.keymap.layers[i][idx as usize];
      if let Behavior::Transparent = behavior {
        continue;
      }
      // TODO: set virtual state
      let action = behavior_to_action(behavior);
      self.key_down_layer[idx as usize] = i as LayerIndex;
      match action {
        Action::SendKey(kui) => {
          self.apply_kui_down(kui);
          return Ok(true);
        }
        Action::Internal(vfunc) => {
          self.apply_vfunc_down(vfunc);
          return Ok(false);
        }
        Action::Nothing => {
          return Ok(false);
        }
      }
    }
    Ok(false)
  }

  fn key_up(&mut self, idx: KeyIndex) -> Result<bool, Error> {
    // TODO: check key mask?
    let layer = self.key_down_layer[idx as usize];
    let behavior = self.keymap.layers[layer as usize][idx as usize];
    let action = behavior_to_action(behavior);
    match action {
      Action::SendKey(kui) => {
        self.apply_kui_up(kui);
        return Ok(true);
      }
      Action::Internal(vfunc) => {
        self.apply_vfunc_up(vfunc);
        return Ok(false);
      }
      Action::Nothing => {
        return Ok(false);
      }
    }
  }

  pub fn update(
    &mut self,
    key_events: Vec<KeyEvent, BUS_WIDTH>)
    -> Result<bool, Error>
  {
    let mut updated = false;
    for event in key_events.into_iter() {
      let now_updated = match event {
        KeyEvent::Down(idx) => self.key_down(idx)?,
        KeyEvent::Up(idx) => self.key_up(idx)?,
      };
      updated = updated || now_updated;
      if self.reset {
        break;
      }
    }
    Ok(updated)
  }

  pub fn get_report<'a>(&'a self) -> &'a NKROBootKeyboardReport {
    &self.usb_report
  }
}

#[derive(Debug,Copy,Clone)]
pub enum KeyEvent {
  Down(KeyIndex),
  Up(KeyIndex),
}
