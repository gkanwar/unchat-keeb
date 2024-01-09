use heapless::Vec;
use usbd_hid::descriptor::generator_prelude::*;

use crate::Error;
use crate::prelude::*;
use crate::vkeyboard::VKeyboard;

// Usb I/O is handled in the board-specific shell
// pub struct Usb {
// }

// impl Usb {
//   pub fn send(&mut self, report: &NKROBootKeyboardReport) -> Result<(), Error> {
//     todo!("usb protocol");
//   }
// }


// from usbd_hid v0.7.0
// in the future, we can replace this with
// use usbd_hid::descriptor::KeyboardUsage;
#[non_exhaustive]
#[repr(u8)]
pub enum KeyboardUsage {
  KeyboardErrorRollOver = 1,
  KeyboardPOSTFail = 2,
  KeyboardErrorUndefined = 3,
  KeyboardAa = 4,
  KeyboardBb = 5,
  KeyboardCc = 6,
  KeyboardDd = 7,
  KeyboardEe = 8,
  KeyboardFf = 9,
  KeyboardGg = 10,
  KeyboardHh = 11,
  KeyboardIi = 12,
  KeyboardJj = 13,
  KeyboardKk = 14,
  KeyboardLl = 15,
  KeyboardMm = 16,
  KeyboardNn = 17,
  KeyboardOo = 18,
  KeyboardPp = 19,
  KeyboardQq = 20,
  KeyboardRr = 21,
  KeyboardSs = 22,
  KeyboardTt = 23,
  KeyboardUu = 24,
  KeyboardVv = 25,
  KeyboardWw = 26,
  KeyboardXx = 27,
  KeyboardYy = 28,
  KeyboardZz = 29,
  Keyboard1Exclamation = 30,
  Keyboard2At = 31,
  Keyboard3Hash = 32,
  Keyboard4Dollar = 33,
  Keyboard5Percent = 34,
  Keyboard6Caret = 35,
  Keyboard7Ampersand = 36,
  Keyboard8Asterisk = 37,
  Keyboard9OpenParens = 38,
  Keyboard0CloseParens = 39,
  KeyboardEnter = 40,
  KeyboardEscape = 41,
  KeyboardBackspace = 42,
  KeyboardTab = 43,
  KeyboardSpacebar = 44,
  KeyboardDashUnderscore = 45,
  KeyboardEqualPlus = 46,
  KeyboardOpenBracketBrace = 47,
  KeyboardCloseBracketBrace = 48,
  KeyboardBackslashBar = 49,
  KeyboardNonUSHash = 50,
  KeyboardSemiColon = 51,
  KeyboardSingleDoubleQuote = 52,
  KeyboardBacktickTilde = 53,
  KeyboardCommaLess = 54,
  KeyboardPeriodGreater = 55,
  KeyboardSlashQuestion = 56,
  KeyboardCapsLock = 57,
  KeyboardF1 = 58,
  KeyboardF2 = 59,
  KeyboardF3 = 60,
  KeyboardF4 = 61,
  KeyboardF5 = 62,
  KeyboardF6 = 63,
  KeyboardF7 = 64,
  KeyboardF8 = 65,
  KeyboardF9 = 66,
  KeyboardF10 = 67,
  KeyboardF11 = 68,
  KeyboardF12 = 69,
  KeyboardPrintScreen = 70,
  KeyboardScrollLock = 71,
  KeyboardPause = 72,
  KeyboardInsert = 73,
  KeyboardHome = 74,
  KeyboardPageUp = 75,
  KeyboardDelete = 76,
  KeyboardEnd = 77,
  KeyboardPageDown = 78,
  KeyboardRightArrow = 79,
  KeyboardLeftArrow = 80,
  KeyboardDownArrow = 81,
  KeyboardUpArrow = 82,
  KeypadNumLock = 83,
  KeypadDivide = 84,
  KeypadMultiply = 85,
  KeypadMinus = 86,
  KeypadPlus = 87,
  KeypadEnter = 88,
  Keypad1End = 89,
  Keypad2DownArrow = 90,
  Keypad3PageDown = 91,
  Keypad4LeftArrow = 92,
  Keypad5 = 93,
  Keypad6RightArrow = 94,
  Keypad7Home = 95,
  Keypad8UpArrow = 96,
  Keypad9PageUp = 97,
  Keypad0Insert = 98,
  KeypadPeriodDelete = 99,
  KeyboardNonUSSlash = 100,
  KeyboardApplication = 101,
  KeyboardPower = 102,
  KeypadEqual = 103,
  KeyboardF13 = 104,
  KeyboardF14 = 105,
  KeyboardF15 = 106,
  KeyboardF16 = 107,
  KeyboardF17 = 108,
  KeyboardF18 = 109,
  KeyboardF19 = 110,
  KeyboardF20 = 111,
  KeyboardF21 = 112,
  KeyboardF22 = 113,
  KeyboardF23 = 114,
  KeyboardF24 = 115,
  KeyboardExecute = 116,
  KeyboardHelp = 117,
  KeyboardMenu = 118,
  KeyboardSelect = 119,
  KeyboardStop = 120,
  KeyboardAgain = 121,
  KeyboardUndo = 122,
  KeyboardCut = 123,
  KeyboardCopy = 124,
  KeyboardPaste = 125,
  KeyboardFind = 126,
  KeyboardMute = 127,
  KeyboardVolumeUp = 128,
  KeyboardVolumeDown = 129,
  KeyboardLockingCapsLock = 130,
  KeyboardLockingNumLock = 131,
  KeyboardLockingScrollLock = 132,
  KeypadComma = 133,
  KeypadEqualSign = 134,
  KeyboardInternational1 = 135,
  KeyboardInternational2 = 136,
  KeyboardInternational3 = 137,
  KeyboardInternational4 = 138,
  KeyboardInternational5 = 139,
  KeyboardInternational6 = 140,
  KeyboardInternational7 = 141,
  KeyboardInternational8 = 142,
  KeyboardInternational9 = 143,
  KeyboardLANG1 = 144,
  KeyboardLANG2 = 145,
  KeyboardLANG3 = 146,
  KeyboardLANG4 = 147,
  KeyboardLANG5 = 148,
  KeyboardLANG6 = 149,
  KeyboardLANG7 = 150,
  KeyboardLANG8 = 151,
  KeyboardLANG9 = 152,
  KeyboardAlternateErase = 153,
  KeyboardSysReqAttention = 154,
  KeyboardCancel = 155,
  KeyboardClear = 156,
  KeyboardPrior = 157,
  KeyboardReturn = 158,
  KeyboardSeparator = 159,
  KeyboardOut = 160,
  KeyboardOper = 161,
  KeyboardClearAgain = 162,
  KeyboardCrSelProps = 163,
  KeyboardExSel = 164,
  Keypad00 = 176,
  Keypad000 = 177,
  ThousandsSeparator = 178,
  DecimalSeparator = 179,
  CurrencyUnit = 180,
  CurrencySubunit = 181,
  KeypadOpenParens = 182,
  KeypadCloseParens = 183,
  KeypadOpenBrace = 184,
  KeypadCloseBrace = 185,
  KeypadTab = 186,
  KeypadBackspace = 187,
  KeypadA = 188,
  KeypadB = 189,
  KeypadC = 190,
  KeypadD = 191,
  KeypadE = 192,
  KeypadF = 193,
  KeypadBitwiseXor = 194,
  KeypadLogicalXor = 195,
  KeypadModulo = 196,
  KeypadLeftShift = 197,
  KeypadRightShift = 198,
  KeypadBitwiseAnd = 199,
  KeypadLogicalAnd = 200,
  KeypadBitwiseOr = 201,
  KeypadLogicalOr = 202,
  KeypadColon = 203,
  KeypadHash = 204,
  KeypadSpace = 205,
  KeypadAt = 206,
  KeypadExclamation = 207,
  KeypadMemoryStore = 208,
  KeypadMemoryRecall = 209,
  KeypadMemoryClear = 210,
  KeypadMemoryAdd = 211,
  KeypadMemorySubtract = 212,
  KeypadMemoryMultiply = 213,
  KeypadMemoryDivide = 214,
  KeypadPositiveNegative = 215,
  KeypadClear = 216,
  KeypadClearEntry = 217,
  KeypadBinary = 218,
  KeypadOctal = 219,
  KeypadDecimal = 220,
  KeypadHexadecimal = 221,
  KeyboardLeftControl = 224,
  KeyboardLeftShift = 225,
  KeyboardLeftAlt = 226,
  KeyboardLeftGUI = 227,
  KeyboardRightControl = 228,
  KeyboardRightShift = 229,
  KeyboardRightAlt = 230,
  KeyboardRightGUI = 231,
  Reserved = 232,
}


const _: () = assert!(USB_CLASS_HID == 3, "USB class must be keyboard");
// NOTE: must be in sync with HID descriptor
const USB_USAGE_MIN: u8 = 0x02;

#[gen_hid_descriptor(
  (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
    (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
      #[packed_bits 8]
      #[item_settings data,variable,absolute]
      modifier = input;
    };
    (usage_min = 0x00, usage_max = 0xFF) = {
      #[item_settings constant,variable,absolute]
      reserved = input;
    };
    (usage_page = LEDS, usage_min = 0x01, usage_max = 0x05) = {
      #[packed_bits 5]
      #[item_settings data,variable,absolute]
      leds = output;
    };
    (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0xDD) = {
      #[item_settings data,array,absolute]
      boot_keys = input;
    };
    (usage_page = KEYBOARD, usage_min = 0x02, usage_max = 0x81) = {
      #[packed_bits 128]
      #[item_settings data,variable,absolute]
      nkro_keys = input;
    };
  }
)]
#[derive(Default)]
pub struct NKROBootKeyboardReport {
  // fixed boot format report
  pub modifier: u8,
  pub reserved: u8,
  pub leds: u8,
  pub boot_keys: [u8; 6],
  // nkro extension for USB-compatible OS
  pub nkro_keys: [u8; 16],
}

pub enum KeyUsageAndIndex {
  Normal {
    // bios usage and nkro index
    usage: u8,
    byte: usize,
    bit: usize,
  },
  Modifier {
    bit: usize,
  },
}
impl KeyUsageAndIndex {
  pub fn new(usage: KeyboardUsage) -> Self {
    let usage_idx: u8 = usage as u8;
    if usage_idx >= NKRO_MIN_KEY && usage_idx <= NKRO_MAX_KEY {
      Self::Normal {
        usage: usage_idx,
        byte: ((usage_idx - USB_USAGE_MIN) / 8) as usize,
        bit: ((usage_idx - USB_USAGE_MIN) % 8) as usize,
      }
    }
    else if usage_idx >= MIN_MODIFIER && usage_idx <= MAX_MODIFIER {
      Self::Modifier {
        bit: (usage_idx - MIN_MODIFIER) as usize,
      }
    }
    else {
      panic!("unsupported usage");
    }
  }
}
