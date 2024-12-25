#![no_std]
#![no_main]

// choice of Board Support Package
// use rp_pico as bsp;
use adafruit_qt_py_rp2040 as bsp;

use bsp::entry;
use bsp::hal::{
  self,
  adc,
  clocks::{init_clocks_and_plls, Clock, ClocksManager},
  pac::{self, interrupt},
  pio::{self, PIOExt},
  gpio,
  sio,
  timer,
  watchdog
};
use cortex_m as cpu;
use cortex_m::interrupt::Mutex;
use ehal::digital::v2::{InputPin, OutputPin, PinState};
use ehal::adc::OneShot;
use ws2812_pio::Ws2812Direct;
use smart_leds::{SmartLedsWrite, RGB8};

use core::convert::Infallible;
use core::panic::PanicInfo;
use core::cell::Cell;
use core::fmt::{self, Write};
use heapless::Vec;

use usb_device::{
  prelude::*,
  class_prelude::*,
  // descriptor::lang_id::LangID,
};
use usbd_hid::{
  hid_class, descriptor::generator_prelude::*
};
use usbd_serial::{
  SerialPort,
  embedded_io::{Write as EWrite, ReadReady},
};

use keeb::{
  prelude::*,
  error::Error,
  layout::Keymap,
  board::Board,
  bus::AnalogBus,
  usb::NKROBootKeyboardReport,
  switch_matrix::SwitchMatrix,
  vkeyboard::VKeyboard,
};

enum NpState {
  Off, Ok, Panic,
}

fn get_np_color(state: NpState) -> RGB8 {
  match state {
    NpState::Off => (0, 0, 0).into(),
    NpState::Ok => (50, 0, 50).into(),
    NpState::Panic => (75, 0, 0).into(),
  }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  // set panic led
  set_neopixel(NpState::Panic);
  // write panic info to serial repeatedly
  let mut buf = WriteBuf::<1024>::new();
  write!(&mut buf, "Panic: {}\r\n", info).ok();
  loop {
    for chunk in buf.data[..].chunks(128) {
      write_serial(chunk);
      drain_serial();
    }
    safe_delay_ms(200);
    set_neopixel(NpState::Off);
    safe_delay_ms(800);
    set_neopixel(NpState::Panic);
  }
}

fn write_serial(buf: &[u8]) {
  cpu::interrupt::free(|cs| {
    let mut usb_interface = Cell::new(None);
    mutex_usb_interface.borrow(cs).swap(&usb_interface);
    match usb_interface.get_mut() {
      Some(UsbInterface{ usb_serial_class, .. }) => {
        usb_serial_class.write(buf).ok();
      },
      _ => {}
    };
    mutex_usb_interface.borrow(cs).swap(&usb_interface);
  });
}

fn write_fmt_serial(fmt: fmt::Arguments) {
  cpu::interrupt::free(|cs| {
    let mut usb_interface = Cell::new(None);
    mutex_usb_interface.borrow(cs).swap(&usb_interface);
    match usb_interface.get_mut() {
      Some(UsbInterface{ usb_serial_class, .. }) => {
        usb_serial_class.write_fmt(fmt).ok();
      },
      _ => {}
    };
    mutex_usb_interface.borrow(cs).swap(&usb_interface);
  });
}

fn drain_serial() {
  cpu::interrupt::free(|cs| {
    let mut buf: [u8; 1024] = [0; 1024];
    let mut usb_interface = Cell::new(None);
    mutex_usb_interface.borrow(cs).swap(&usb_interface);
    match usb_interface.get_mut() {
      Some(UsbInterface{ usb_serial_class, .. }) => {
        usb_serial_class.read(&mut buf[..]).ok();
      },
      _ => {}
    };
    mutex_usb_interface.borrow(cs).swap(&usb_interface);
  });
}

fn grab_delay() -> Cell<Option<cpu::delay::Delay>> {
  cpu::interrupt::free(|cs| {
    let delay_cell = Cell::new(None);
    mutex_delay.borrow(cs).swap(&delay_cell);
    return delay_cell;
  })
}

fn return_delay(delay: Cell<Option<cpu::delay::Delay>>) {
  cpu::interrupt::free(|cs| {
    mutex_delay.borrow(cs).swap(&delay);
  });
}

fn safe_delay_ms(ms: u32) {
  let mut delay_cell = grab_delay();
  match delay_cell.get_mut() {
    Some(delay) => delay.delay_ms(ms),
    None => panic!("Missing delay timer"),
  };
  return_delay(delay_cell);
}

fn set_neopixel(state: NpState) {
  cpu::interrupt::free(|cs| {
    let mut neopixel = Cell::new(None);
    mutex_neopixel.borrow(cs).swap(&neopixel);
    match neopixel.get_mut() {
      Some(Neopixel { power, control }) => {
        control.write([get_np_color(state)].iter().copied()).unwrap_or(());
      },
      _ => {}
    }
    mutex_neopixel.borrow(cs).swap(&neopixel);
  });
}

type PinOut = gpio::FunctionSio<gpio::SioOutput>;
type PinIn = gpio::FunctionSio<gpio::SioInput>;
type PinPD = gpio::PullDown;
type PinFloat = gpio::PullNone;
type PinA0 = gpio::Pin<gpio::bank0::Gpio29, PinIn, PinFloat>;
type PinA1 = gpio::Pin<gpio::bank0::Gpio28, PinIn, PinFloat>;
type PinA2 = gpio::Pin<gpio::bank0::Gpio27, PinIn, PinFloat>;
type PinA3 = gpio::Pin<gpio::bank0::Gpio26, PinIn, PinFloat>;

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
      Ok(pin) => Ok(Self::Pin { pin }),
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
      Ok(pin) => Ok(Self::Pin { pin }),
      Err(_) => Err(Error::PinConfigError)
    }
  }
}

type NeopixelControlPin = gpio::Pin<gpio::DynPinId, gpio::FunctionPio0, gpio::PullNone>;
struct Neopixel {
  power: GpioOut,
  control: Ws2812Direct<pac::PIO0, pio::SM0, NeopixelControlPin>,
}

struct UserPins {
  neopixel: Neopixel,
  analog_pins: (PinA0, PinA1, PinA2, PinA3),
  general_pins: Vec<GpioIn, 32>,
  general_ids: Vec<usize, 32>,
}

fn into_user_pins<'timer, C: Clock>(
  pins: hal::gpio::bank0::Pins, pio0: &mut pio::PIO<pac::PIO0>,
  sm0: pio::UninitStateMachine<pio::PIO0SM0>, clock: &C)
  -> UserPins
{
  let neopixel = Neopixel {
    power: GpioOut {
      pin: pins.gpio11.into_push_pull_output_in_state(gpio::PinState::High).into_dyn_pin()
    },
    control: Ws2812Direct::new(
      pins.gpio12.into_function().into_pull_type::<gpio::PullNone>().into_dyn_pin(),
      pio0,
      sm0,
      clock.freq(),
    ),
  };
  UserPins {
    neopixel,
    analog_pins: (
      pins.gpio29.into_floating_input(),
      pins.gpio28.into_floating_input(),
      pins.gpio27.into_floating_input(),
      pins.gpio26.into_floating_input(),
    ),
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
      GpioIn::new(pins.gpio23),
      GpioIn::new(pins.gpio24),
      GpioIn::new(pins.gpio25),
    ].into_iter().collect(),
    general_ids: [
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 13, 14, 15, 16,
      17, 18, 19, 20, 21, 22, 23, 24, 25,
    ].into_iter().collect(),
  }
}

struct AdcBus {
  pins: (adc::AdcPin<PinA0>, adc::AdcPin<PinA1>, adc::AdcPin<PinA2>, adc::AdcPin<PinA3>),
  adc: adc::Adc,
}

impl AnalogBus for AdcBus {
  fn read(&mut self) -> RegValue {
    [
      self.adc.read(&mut self.pins.0).unwrap_or(0),
      self.adc.read(&mut self.pins.1).unwrap_or(0),
      self.adc.read(&mut self.pins.2).unwrap_or(0),
      self.adc.read(&mut self.pins.3).unwrap_or(0),
    ]
  }
}

type UsbBusAlloc = UsbBusAllocator<hal::usb::UsbBus>;
type UsbDev<'a> = UsbDevice<'a, hal::usb::UsbBus>;
type UsbKbdClass<'a> = hid_class::HIDClass<'a, hal::usb::UsbBus>;
type UsbSerialClass<'a> = SerialPort<'a, hal::usb::UsbBus>;
struct UsbInterface<'a> {
  usb_dev: UsbDev<'a>,
  usb_kbd_class: UsbKbdClass<'a>,
  usb_serial_class: UsbSerialClass<'a>,
}

static mutex_usb_interface: Mutex<Cell<Option<UsbInterface>>>
  = Mutex::new(Cell::new(None));
static mutex_neopixel: Mutex<Cell<Option<Neopixel>>>
  = Mutex::new(Cell::new(None));
// delay timer must be global for panic handler
static mutex_delay: Mutex<Cell<Option<cpu::delay::Delay>>>
  = Mutex::new(Cell::new(None));

#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
  cpu::interrupt::free(|cs| {
    let mut usb_interface = Cell::new(None);
    mutex_usb_interface.borrow(cs).swap(&usb_interface);
    match usb_interface.get_mut() {
      Some(UsbInterface{ usb_dev, usb_kbd_class, usb_serial_class, .. }) => {
        usb_dev.poll(&mut [usb_kbd_class, usb_serial_class]);
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
    &mut pac.RESETS, &mut watchdog).unwrap();
  let delay = cpu::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
  let mut delay_cell = Cell::new(Some(delay));
  return_delay(delay_cell);
  let adc = adc::Adc::new(pac.ADC, &mut pac.RESETS);
  let pins = hal::gpio::bank0::Pins::new(
    pac.IO_BANK0,
    pac.PADS_BANK0,
    sio.gpio_bank0,
    &mut pac.RESETS,
  );
  let (mut pio0, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
  let timer = timer::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

  // split up pins, init neopixel for diagnostics
  let mut user_pins = into_user_pins(pins, &mut pio0, sm0, &clocks.peripheral_clock);
  user_pins.neopixel.control.write([get_np_color(NpState::Ok)].iter().copied()).unwrap();
  let neopixel = Cell::new(Some(user_pins.neopixel));
  cpu::interrupt::free(|cs| {
    mutex_neopixel.borrow(cs).swap(&neopixel);
  });

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
  let usb_serial = SerialPort::new(&usb_bus);
  let str_desc = StringDescriptors::new(LangID::EN)
    .manufacturer("gkanwar")
    .product("Unchat-40")
    .serial_number("XXXX");
  let usb_dev =
    UsbDeviceBuilder::new(&usb_bus, USB_VID_PID_GEN_KBD)
    .strings(&[str_desc]).map_err(|_| Error::UsbError).unwrap()
    // .composite_with_iads()
    .device_class(0x00) // composite
    .build();
  let usb_interface = Cell::new(Some(UsbInterface {
    usb_dev, usb_kbd_class, usb_serial_class: usb_serial
  }));
  cpu::interrupt::free(|cs| {
    mutex_usb_interface.borrow(cs).swap(&usb_interface);
  });
  // USB interrupts
  unsafe {
    pac::NVIC::unmask(pac::Interrupt::USBCTRL_IRQ);
  }

  write_serial(b"Unchat-40 firmware loading...\r\n");

  // load keymap
  let (keymap, _bytes_read): (Keymap, usize) =
    serde_json::from_slice(include_bytes!("../../keymaps/split-40-colemak-callum.json"))
    .unwrap();
  let layout = keeb::layout::get_layout(keymap.layout);
  write_serial(b"Loaded keymap.\r\n");

  // load board
  let (board, _bytes_read): (Board, usize) =
    serde_json::from_slice(include_bytes!("../../boards/unchat-40.json"))
    .unwrap();
  write_serial(b"Loaded board config.\r\n");

  let board_pins =
    keeb::board::split_pins(user_pins.general_pins, user_pins.general_ids, &board).unwrap();
  if user_pins.analog_pins.0.id().num != board.bus_pins[0]
    || user_pins.analog_pins.1.id().num != board.bus_pins[1]
    || user_pins.analog_pins.2.id().num != board.bus_pins[2]
    || user_pins.analog_pins.3.id().num != board.bus_pins[3] {
      panic!("Board bus pins mismatch");
  }
  let mut bus = AdcBus {
    pins: (
      adc::AdcPin::new(user_pins.analog_pins.0).unwrap_or_else(|_| panic!()),
      adc::AdcPin::new(user_pins.analog_pins.1).unwrap_or_else(|_| panic!()),
      adc::AdcPin::new(user_pins.analog_pins.2).unwrap_or_else(|_| panic!()),
      adc::AdcPin::new(user_pins.analog_pins.3).unwrap_or_else(|_| panic!()),
    ),
    adc,
  };

  let reg_map = keeb::board::make_reg_map(&board, &layout);
  let mut switches = SwitchMatrix::<GpioOut>::new(
    reg_map.clone(), board_pins.sel_pins).unwrap();
  let mut vkbd = VKeyboard::new(keymap).unwrap();
  write_serial(b"Established switch matrix and virtual keyboard.\r\n");
  write_serial(b"Running main loop.\r\n");

  let mut buf: [u8; 64] = [0; 64]; // for usb OUT packets
  let mut pending = false;
  loop {
    let mut delay_cell = grab_delay();
    let delay = match delay_cell.get_mut() {
      Some(delay) => delay,
      None => panic!("missing delay"),
    };
    delay.delay_ms(1);
    // let mut log_buf = WriteBuf::<4196>::new();
    let (updated, new_bus) = keeb::tick(
      bus, &mut switches, &mut vkbd, delay, |args: fmt::Arguments<'_>| {
        // write_fmt_serial(args);
      }
    ).unwrap();
    // write_serial(&log_buf.data[..]);
    bus = new_bus;
    return_delay(delay_cell);

    if vkbd.reset {
      hal::rom_data::reset_to_usb_boot(0, 0);
    }

    let report = if updated || pending {
      // write_fmt_serial(format_args!("Kbd keys: {:?}\r\n", vkbd.get_report().nkro_keys));
      Some(vkbd.get_report().clone())
    }
    else {
      None
    };

    
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
          Some(UsbInterface{ usb_kbd_class, usb_serial_class, .. }) => {
            match usb_kbd_class.pull_raw_output(&mut buf) {
              Ok(size) => {},
              Err(UsbError::WouldBlock) => {}, // no data
              Err(err) => panic!("unexpected read error"),
            }
            match report {
              Some(mut report) => {
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
              }
              None => {}
            }
            // get serial input
            if usb_serial_class.read_ready().unwrap_or(false) {
              let mut buf: [u8; 128] = [0; 128];
              match usb_serial_class.read(&mut buf[..]) {
                Ok(n) => {
                  // FORNOW: echo messages
                  usb_serial_class.write(&buf[..n]).ok();
                  usb_serial_class.flush().ok();
                }
                Err(UsbError::WouldBlock) => {}
                Err(_) => {
                  usb_serial_class.write(b"X\r\n").ok();
                }
              }
            }
          },
          None => {},
        }
      }
      mutex_usb_interface.borrow(cs).swap(&usb_interface);
    });
  }

  // // pause, then reboot into BOOTSEL
  // safe_delay_ms(1000);
  // hal::rom_data::reset_to_usb_boot(0, 0);
  // unreachable!();
}
