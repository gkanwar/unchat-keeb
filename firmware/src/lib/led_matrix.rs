use ehal::digital::v2::OutputPin;
use heapless::Vec;

use crate::board::RegMap;
use crate::bus::OutputBus;
use crate::consts::*;

pub struct LedMatrix<Q: OutputPin> {
  reg_map: RegMap, // TODO: is this the right information to hang on to?
  reg_rst_pins: Vec<Q, MAX_REGS>,
  reg_clk_pins: Vec<Q, MAX_REGS>,
}

impl<Q: OutputPin> LedMatrix<Q> {
  pub fn tick(&mut self, bus: &mut OutputBus<Q>) {

  }
}
