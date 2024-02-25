#![no_std]
#![no_main]

// choice of Board Support Package
use rp_pico as bsp;

use bsp::entry;
use bsp::hal::{
  self,
  clocks::{init_clocks_and_plls, Clock},
  pac::{self, interrupt},
  gpio,
  sio,
  watchdog
};
use cortex_m as cpu;
use cortex_m::interrupt::Mutex;
use ehal::digital::v2::{InputPin, OutputPin, PinState};

use core::convert::Infallible;
use core::panic::PanicInfo;
use core::cell::Cell;
use heapless::Vec;

use usb_device::{
  prelude::*,
  class_prelude::*,
};
use usbd_hid::{
  hid_class, descriptor::generator_prelude::*
};

use keeb::{
  prelude::*,
  Error,
  layout::{Keymap},
  board::{Board},
  bus::{TryIntoInputPin, TryIntoOutputPin},
  usb::NKROBootKeyboardReport,
  switch_matrix::SwitchMatrix,
  led_matrix::LedMatrix,
  vkeyboard::VKeyboard,
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  // set panic led
  set_led(true);
  hal::halt();
}

fn set_led(state: bool) {
  cpu::interrupt::free(|cs| {
    let mut led_pin = Cell::new(None);
    mutex_led_pin.borrow(cs).swap(&led_pin);
    match led_pin.get_mut() {
      Some(pin) => {
        if state {
          pin.set_high().unwrap();
        }
        else {
          pin.set_low().unwrap();
        }
      },
      _ => {}
    }
    mutex_led_pin.borrow(cs).swap(&led_pin);
  });
}

type PinOut = gpio::FunctionSio<gpio::SioOutput>;
type PinIn = gpio::FunctionSio<gpio::SioInput>;
type PinPD = gpio::PullDown;

struct GpioIn {
  pin: gpio::Pin<gpio::DynPinId, PinIn, PinPD>
}
impl GpioIn {
  fn new<I: gpio::PinId, F: gpio::Function, P: gpio::PullType>(
    pin: gpio::Pin<I, F, P>) -> Self
  where I: gpio::ValidFunction<PinIn> {
    Self {
      pin: pin.into_pull_down_input().into_dyn_pin()
    }
  }
}

struct GpioOut {
  pin: gpio::Pin<gpio::DynPinId, PinOut, PinPD>
}
impl GpioOut {
  fn new<I: gpio::PinId, F: gpio::Function, P: gpio::PullType>(
    pin: gpio::Pin<I, F, P>) -> Self
  where I: gpio::ValidFunction<PinOut> {
    Self {
      pin: pin.into_push_pull_output().into_pull_type().into_dyn_pin()
    }
  }
}

impl InputPin for GpioIn {
  type Error = Infallible;
  fn is_high(&self) -> Result<bool, Self::Error> {
    self.pin.is_high()
  }
  fn is_low(&self) -> Result<bool, Self::Error> {
    self.pin.is_low()
  }
}

impl TryIntoOutputPin for GpioIn {
  type Pin = GpioOut;
  fn try_into_output_pin(self) -> Result<Self::Pin, Error> {
    match self.pin.try_into_function::<PinOut>() {
      Ok(pin) => Ok(Self::Pin { pin: pin }),
      Err(_) => Err(Error::PinConfigError)
    }
  }
}

impl OutputPin for GpioOut {
  type Error = Infallible;
  fn set_low(&mut self) -> Result<(), Self::Error> {
    self.pin.set_low()
  }
  fn set_high(&mut self) -> Result<(), Self::Error> {
    self.pin.set_high()
  }
}


impl TryIntoInputPin for GpioOut {
  type Pin = GpioIn;
  fn try_into_input_pin(self) -> Result<Self::Pin, Error> {
    match self.pin.try_into_function::<PinIn>() {
      Ok(pin) => Ok(Self::Pin { pin: pin }),
      Err(_) => Err(Error::PinConfigError)
    }
  }
}

struct UserPins {
  led: GpioOut,
  general_pins: Vec<GpioIn, 32>,
  general_ids: Vec<usize, 32>,
}

fn into_user_pins(pins: bsp::Pins) -> UserPins {
  UserPins {
    led: GpioOut::new(pins.led),
    general_pins: [
      GpioIn::new(pins.gpio0),
      GpioIn::new(pins.gpio1),
      GpioIn::new(pins.gpio2),
      GpioIn::new(pins.gpio3),
      GpioIn::new(pins.gpio4),
      GpioIn::new(pins.gpio5),
      GpioIn::new(pins.gpio6),
      GpioIn::new(pins.gpio7),
      GpioIn::new(pins.gpio8),
      GpioIn::new(pins.gpio9),
      GpioIn::new(pins.gpio10),
      GpioIn::new(pins.gpio11),
      GpioIn::new(pins.gpio12),
      GpioIn::new(pins.gpio13),
      GpioIn::new(pins.gpio14),
      GpioIn::new(pins.gpio15),
      GpioIn::new(pins.gpio16),
      GpioIn::new(pins.gpio17),
      GpioIn::new(pins.gpio18),
      GpioIn::new(pins.gpio19),
      GpioIn::new(pins.gpio20),
      GpioIn::new(pins.gpio21),
      GpioIn::new(pins.gpio22),
      GpioIn::new(pins.gpio26),
      GpioIn::new(pins.gpio27),
      GpioIn::new(pins.gpio28),
    ].into_iter().collect(),
    general_ids: [
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
      11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
      21, 22, 26, 27, 28
    ].into_iter().collect(),
  }
}

type UsbBusAlloc = UsbBusAllocator<hal::usb::UsbBus>;
type UsbDev<'a> = UsbDevice<'a, hal::usb::UsbBus>;
type UsbKbdClass<'a> = hid_class::HIDClass<'a, hal::usb::UsbBus>;
struct UsbInterface<'a> {
  usb_dev: UsbDev<'a>,
  usb_kbd_class: UsbKbdClass<'a>,
}

static mutex_usb_interface: Mutex<Cell<Option<UsbInterface>>>
  = Mutex::new(Cell::new(None));
static mutex_led_pin: Mutex<Cell<Option<GpioOut>>>
  = Mutex::new(Cell::new(None));

#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
  cpu::interrupt::free(|cs| {
    let mut usb_interface = Cell::new(None);
    mutex_usb_interface.borrow(cs).swap(&usb_interface);
    match usb_interface.get_mut() {
      Some(UsbInterface{ usb_dev, usb_kbd_class, .. }) => {
        usb_dev.poll(&mut [usb_kbd_class]);
      },
      _ => {}
    }
    mutex_usb_interface.borrow(cs).swap(&usb_interface);
  });
}

// generic keyboard
// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const USB_VID_PID_GEN_KBD: UsbVidPid = UsbVidPid(0x16c0, 0x27db);
// USB poll bInterval [1-255]
const USB_POLL_MS: u8 = 1;

#[entry]
fn rp2040_main() -> ! {
  // usb bus must be static lifetime for interrupts
  static mut USB_BUS: Option<UsbBusAlloc> = None;

  // init board state and components
  let mut pac = pac::Peripherals::take().unwrap();
  let core = pac::CorePeripherals::take().unwrap();
  let mut watchdog = watchdog::Watchdog::new(pac.WATCHDOG);
  let sio = sio::Sio::new(pac.SIO);
  const XTAL_FREQ_HZ: u32 = 12_000_000_u32;
  let clocks = init_clocks_and_plls(
    XTAL_FREQ_HZ, pac.XOSC, pac.CLOCKS, pac.PLL_SYS, pac.PLL_USB,
    &mut pac.RESETS, &mut watchdog).ok().unwrap();
  let mut delay = cpu::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
  let pins = bsp::Pins::new(
    pac.IO_BANK0,
    pac.PADS_BANK0,
    sio.gpio_bank0,
    &mut pac.RESETS,
  );

  // set up USB
  *USB_BUS = Some(UsbBusAllocator::new(hal::usb::UsbBus::new(
    pac.USBCTRL_REGS, pac.USBCTRL_DPRAM, clocks.usb_clock, true, &mut pac.RESETS
  )));
  let usb_bus = USB_BUS.as_ref().unwrap();
  let desc = NKROBootKeyboardReport::desc();
  let usb_kbd_class = hid_class::HIDClass::new_with_settings(
    &usb_bus, desc, USB_POLL_MS,
    hid_class::HidClassSettings {
      subclass: hid_class::HidSubClass::NoSubClass,
      protocol: hid_class::HidProtocol::Keyboard,
      config: hid_class::ProtocolModeConfig::DefaultBehavior,
      locale: hid_class::HidCountryCode::US,
    });
  let usb_dev =
    UsbDeviceBuilder::new(&usb_bus, USB_VID_PID_GEN_KBD)
    .manufacturer("gkanwar")
    .product("Unchat-42")
    .serial_number("XXXX")
    .build();
  // TODO: OSX doesn't recognize keyboard when HID device class is set
  // .device_class(USB_CLASS_HID)
  let usb_interface = Cell::new(Some(UsbInterface {
    usb_dev, usb_kbd_class
  }));
  cpu::interrupt::free(|cs| {
    mutex_usb_interface.borrow(cs).swap(&usb_interface);
  });
  // USB interrupts
  unsafe {
    pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ);
  }


  // load keymap
  let (keymap, _bytes_read): (Keymap, usize) =
    serde_json::from_slice(include_bytes!("../../keymaps/split-42-colemak.json"))
    .unwrap();
  let layout = keeb::layout::get_layout(keymap.layout);

  // load board
  let (board, _bytes_read): (Board, usize) =
    serde_json::from_slice(include_bytes!("../../boards/unchat-42.json"))
    .unwrap();

  let user_pins = into_user_pins(pins);
  let led_pin = Cell::new(Some(user_pins.led));
  cpu::interrupt::free(|cs| {
    mutex_led_pin.borrow(cs).swap(&led_pin);
  });
  let mut general_pins = user_pins.general_pins;
  let mut general_ids = user_pins.general_ids;

  let board_pins =
    keeb::board::split_pins(general_pins, general_ids, &board).unwrap();
  let (mut in_bus, mut bus_lock) =
    keeb::bus::make_bus(board_pins.bus_pins);

  let reg_map = keeb::board::make_reg_map(&board, &layout);
  let mut switches = SwitchMatrix::<GpioOut>::new(
    reg_map.clone(), board_pins.switch_reg_pins).unwrap();
  let mut leds = LedMatrix::<GpioOut>::new(
    reg_map,
    board_pins.backlight_reg_pins,
    board_pins.backlight_reset_pin,
    board_pins.backlight_dim_pin);
  let mut vkbd = VKeyboard::new(keymap).unwrap();

  let mut buf: [u8; 64] = [0; 64]; // for usb OUT packets
  let mut pending = false;
  loop {
    delay.delay_ms(1);
    let (updated, new_in_bus, new_bus_lock) = keeb::tick(
      in_bus, bus_lock, &mut switches, &mut leds, &mut vkbd, &mut delay
    ).unwrap();
    in_bus = new_in_bus;
    bus_lock = new_bus_lock;
    if vkbd.reset {
      hal::rom_data::reset_to_usb_boot(0, 0);
    }
    if !updated && !pending {
      continue;
    }
    let mut report = vkbd.get_report().clone();
    cpu::interrupt::free(|cs| {
      let mut usb_interface = Cell::new(None);
      mutex_usb_interface.borrow(cs).swap(&usb_interface);
      let configured = match usb_interface.get_mut() {
        Some(UsbInterface{ usb_dev, .. }) =>
          usb_dev.state() == UsbDeviceState::Configured,
        None => false,
      };
      if configured {
        match usb_interface.get_mut() {
          Some(UsbInterface{ usb_kbd_class, .. }) => {
            match usb_kbd_class.pull_raw_output(&mut buf) {
              Ok(size) => {},
              Err(UsbError::WouldBlock) => {}, // no data
              Err(err) => panic!("unexpected read error"),
            }
            match usb_kbd_class.get_protocol_mode() {
              Ok(hid_class::HidProtocolMode::Report) => {
                for i in 0..report.boot_keys.len() {
                  report.boot_keys[i] = 0;
                }
              }
              _ => {}
            }
            match usb_kbd_class.push_input(&report) {
              Ok(size) => {
                pending = false;
              },
              Err(UsbError::WouldBlock) => { // buffer full
                pending = true;
              },
              // usbd-hid bug: error on protocol `Report` with subclass `Boot`
              // hack: forcibly set protocol mode to Boot :(
              Err(UsbError::InvalidState) => {
                pending = true;
                let protocol = usb_kbd_class.get_protocol_mode();
                if let Ok(protocol) = protocol {
                  usb_kbd_class.set_protocol_mode(
                    protocol, hid_class::ProtocolModeConfig::ForceBoot).unwrap();
                }
              },
              Err(err) => panic!("unexpected write error"),
            }
          },
          None => {},
        }
      }
      mutex_usb_interface.borrow(cs).swap(&usb_interface);
    });
  }

  // pause, then reboot into BOOTSEL
  delay.delay_ms(1000);
  hal::rom_data::reset_to_usb_boot(0, 0);
  unreachable!();
}
