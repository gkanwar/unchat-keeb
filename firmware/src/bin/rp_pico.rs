#![no_std]
#![no_main]

// choice of Board Support Package
use rp_pico as bsp;

use bsp::entry;
use bsp::hal::{
  clocks::{init_clocks_and_plls, Clock},
  pac,
  sio,
  watchdog
};
use cortex_m::delay;
use hal::digital::v2::OutputPin;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  // TODO: set panic LED
  loop {} // halt
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

  loop {
    led_pin.set_high().unwrap();
    delay.delay_ms(200);
    led_pin.set_low().unwrap();
    delay.delay_ms(800);
  }
}
