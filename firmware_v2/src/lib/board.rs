use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::Error as DError;
use serde::ser::Error as SError;
use core::result::Result;
use heapless::Vec;
use core::str;

use ehal::digital::v2::{InputPin, OutputPin};

use crate::layout::Layout;
use crate::prelude::*;

type VMatrix<T> = Vec<Vec<T, MAX_COLS>, MAX_ROWS>;
type Matrix<T> = [[T; MAX_COLS]; MAX_ROWS];

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Polarity {N, S}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SwitchSettings {
  pub polarity: Polarity,
  pub trig_down: f32,
  pub trig_up: f32,
}

impl Default for SwitchSettings {
  fn default() -> Self {
    Self {
      polarity: Polarity::S,
      trig_down: 0.1,
      trig_up: 0.4,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Board {
  // matrix (i,j) -> register (idx, bit)
  pub matrix: VMatrix<(RegIndex,RegBitIndex)>,
  // matrix (i,j) -> (polarity, trig_down, trig_up)
  pub matrix_calibration: VMatrix<SwitchSettings>,
  pub bus_pins: [PinIndex; BUS_WIDTH],
  pub sel_pins: [PinIndex; SEL_WIDTH],
  pub led_ind_pins: [PinIndex; 2],
}

pub struct BoardPins<Q: OutputPin> {
  pub sel_pins: [Q; SEL_WIDTH],
  pub led_ind_pins: [Q; 2],
}
pub fn split_pins<P: InputPin, Q: OutputPin, const N: usize>(
  mut pins: Vec<P, N>, mut ids: Vec<usize, N>, board: &Board
) -> Result<BoardPins<Q>, Error>
where P: TryIntoOutputPin<Pin=Q>
{
  let sel_pins: [Q; SEL_WIDTH] =
    board.sel_pins.iter().map(|p| {
      let idx = ids.iter().position(|&i| i == *p as usize)
        .ok_or(Error::PinConfigError)?;
      ids.swap_remove(idx);
      Ok(pins.swap_remove(idx).try_into_output_pin()?)
    }).collect::<Result<Vec<Q, SEL_WIDTH>, Error>>()?
    .into_array().map_err(|_| Error::PinConfigError)?;
  let led_ind_pins: [Q; 2] =
    board.led_ind_pins.iter().map(|p| {
      let idx = ids.iter().position(|&i| i == *p as usize)
        .ok_or(Error::PinConfigError)?;
      ids.swap_remove(idx);
      Ok(pins.swap_remove(idx).try_into_output_pin()?)
    }).collect::<Result<Vec<Q, 2>, Error>>()?
    .into_array().map_err(|_| Error::PinConfigError)?;
  Ok(BoardPins {
    sel_pins, led_ind_pins,
  })
}

#[derive(Clone)]
pub struct RegMap {
  pub regs: Vec<[Option<KeyIndex>; BUS_WIDTH], MAX_REGS>,
  pub calibration: Vec<[SwitchSettings; BUS_WIDTH], MAX_REGS>,
}

pub fn make_reg_map(board: &Board, layout: &Layout) -> RegMap {
  let mut key_matrix: Matrix::<KeyIndex> = [[0; MAX_COLS]; MAX_ROWS];
  for idx in 0..layout.matrix_pos.len() {
    let (i,j) = layout.matrix_pos[idx];
    key_matrix[i][j] = idx as KeyIndex;
  }
  let mut regs: Vec<[Option<KeyIndex>; BUS_WIDTH], MAX_REGS> = Vec::new();
  let mut calibration: Vec<[SwitchSettings; BUS_WIDTH], MAX_REGS> = Vec::new();
  let matrix = &board.matrix;
  for i in 0..matrix.len() {
    let row = &matrix[i];
    for j in 0..row.len() {
      let (reg_idx, reg_bit) = row[j];
      if reg_idx as usize >= regs.capacity() {
        // register higher than MAX_REGS, just skip for now
        continue;
      }
      if reg_idx as usize >= regs.len() {
        // will have capacity per previous check
        regs.resize_default((reg_idx+1) as usize).unwrap();
        calibration.resize_default((reg_idx+1) as usize).unwrap();
      }
      regs[reg_idx as usize][reg_bit as usize] = Some(key_matrix[i][j]);
      calibration[reg_idx as usize][reg_bit as usize] = board.matrix_calibration[i][j];
    }
  }
  assert!(regs.len() == calibration.len());
  return RegMap { regs, calibration };
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn can_deser_board() -> Result<(), String> {
    let (_board, _bytes_read): (Board, usize) =
      serde_json::from_slice(include_bytes!("../../boards/unchat-40.json"))
      .map_err(|e| format!("{}", e))?;
    Ok(())
  }
}
