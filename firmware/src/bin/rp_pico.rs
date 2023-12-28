#![no_std]
#![no_main]

// choice of Board Support Package
use rp_pico as bsp;

use bsp::entry;
use bsp::hal::{
  self,
  clocks::{init_clocks_and_plls, Clock},
  pac,
  gpio,
  sio,
  watchdog
};
use cortex_m::delay;
use ehal::digital::v2::{InputPin, OutputPin, PinState};

use core::convert::Infallible;
use core::panic::PanicInfo;

use keeb::{
  Error,
  bus::{TryIntoInputPin, TryIntoOutputPin}
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  // TODO: set panic LED
  hal::halt();
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

#[entry]
fn rp2040_main() -> ! {
  let mut pac = pac::Peripherals::take().unwrap();
  let core = pac::CorePeripherals::take().unwrap();
  let mut watchdog = watchdog::Watchdog::new(pac.WATCHDOG);
  let sio = sio::Sio::new(pac.SIO);

  const XTAL_FREQ_HZ: u32 = 12_000_000_u32;
  let clocks = init_clocks_and_plls(
    XTAL_FREQ_HZ, pac.XOSC, pac.CLOCKS, pac.PLL_SYS, pac.PLL_USB, &mut
    pac.RESETS, &mut watchdog).ok().unwrap();

  let mut delay = delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

  let pins = bsp::Pins::new(
    pac.IO_BANK0,
    pac.PADS_BANK0,
    sio.gpio_bank0,
    &mut pac.RESETS,
  );

  let mut led_pin = pins.led.into_push_pull_output();

  let mut in_bus = keeb::bus::InputBus {
    pins: [
      GpioIn::new(pins.gpio0),
      GpioIn::new(pins.gpio1),
      GpioIn::new(pins.gpio2),
      GpioIn::new(pins.gpio3),
      GpioIn::new(pins.gpio4),
      GpioIn::new(pins.gpio5),
    ]
  };
  let mut out_bus = in_bus.into_output_bus();

  loop {
    out_bus.set_state([
      PinState::Low,
      PinState::High,
      PinState::High,
      PinState::Low,
      PinState::Low,
      PinState::High
    ]);
    led_pin.set_high().unwrap();
    delay.delay_ms(200);

    in_bus = out_bus.into_input_bus();
    led_pin.set_low().unwrap();
    delay.delay_ms(800);
    out_bus = in_bus.into_output_bus();
  }
}
