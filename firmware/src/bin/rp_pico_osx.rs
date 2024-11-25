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

use usbd_human_interface_device::page::Keyboard;
use usbd_human_interface_device::device::keyboard::{KeyboardLedsReport, BootKeyboardConfig};
use usbd_human_interface_device::prelude::*;


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  hal::halt();
}

// generic keyboard
// https://github.com/obdev/v-usb/blob/master/usbdrv/USB-IDs-for-free.txt
const USB_VID_PID_GEN_KBD: UsbVidPid = UsbVidPid(0x16c0, 0x27db);
// USB poll bInterval [1-255]
const USB_POLL_MS: u8 = 1;

#[entry]
fn rp2040_main() -> ! {

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


  let mut led = pins.led.into_push_pull_output();

  let usb_bus = hal::usb::UsbBus::new(
    pac.USBCTRL_REGS, pac.USBCTRL_DPRAM, clocks.usb_clock, true, &mut pac.RESETS
  );
  let usb_alloc = UsbBusAllocator::new(usb_bus);

  let mut keyboard = UsbHidClassBuilder::new()
    .add_device(
      BootKeyboardConfig::default(),
    )
    .build(&usb_alloc);

  let mut usb_dev = UsbDeviceBuilder::new(&usb_alloc, UsbVidPid(0x1209, 0x0001))
    .manufacturer("usbd-human-interface-device")
    .product("NKRO Keyboard")
    .serial_number("TEST")
    .build();

  let mut i = 0;
  let mut j = 0;
  loop {
    let keys: Vec<Keyboard, 6> = if i >= 100 && i <= 400 {
      led.set_high().unwrap();
      match j {
        0 => [Keyboard::U].into_iter().collect(),
        1 => [Keyboard::U, Keyboard::V].into_iter().collect(),
        2 => [Keyboard::U, Keyboard::V, Keyboard::W].into_iter().collect(),
        3 => [Keyboard::U, Keyboard::V, Keyboard::W, Keyboard::X].into_iter().collect(),
        4 => [Keyboard::U, Keyboard::V, Keyboard::W, Keyboard::X, Keyboard::Y].into_iter().collect(),
        _ => [Keyboard::U, Keyboard::V, Keyboard::W, Keyboard::X, Keyboard::Y, Keyboard::Z].into_iter().collect(),
      }
    } else {
      led.set_low().unwrap();
      [Keyboard::NoEventIndicated].into_iter().collect()
    };
    i += 1;
    if i >= 2000 {
      i = 0;
      j += 1;
    }
    if j >= 6 {
      j = 0;
      hal::rom_data::reset_to_usb_boot(0, 0);
    }

    if i % 8 == 0 { // reports every 8ms
      keyboard.device().write_report(keys).ok();
    }

    // tick once per ms/at 1kHz
    delay.delay_ms(1);
    match keyboard.tick() {
      Ok(l) => {}
      _ => {}
    }

    if usb_dev.poll(&mut [&mut keyboard]) {
      match keyboard.device().read_report() {

        Ok(l) => {}
        _ => {}

      }
    }
  }

  // pause, then reboot into BOOTSEL
  delay.delay_ms(1000);
  hal::rom_data::reset_to_usb_boot(0, 0);
  unreachable!();
}
