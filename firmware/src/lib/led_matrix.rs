use ehal::digital::v2::OutputPin;
use heapless::Vec;

use crate::board::RegMap;
use crate::bus::OutputBus;
use crate::prelude::*;

pub struct LedMatrix<Q: OutputPin> {
  reg_map: RegMap, // TODO: is this the right information to hang on to?
  reg_clk_pins: Vec<Q, MAX_REGS>,
  led_rst_pin: Q,
  led_dim_pin: Q,
}

impl<Q: OutputPin> LedMatrix<Q> {
  pub fn new(
    reg_map: RegMap, reg_clk_pins: Vec<Q, MAX_REGS>,
    led_rst_pin: Q, led_dim_pin: Q) -> Self
  {
    Self { reg_map, reg_clk_pins, led_rst_pin, led_dim_pin }
  }
  pub fn tick(&mut self, bus: &mut OutputBus<Q>) {
    // TODO
  }
}
