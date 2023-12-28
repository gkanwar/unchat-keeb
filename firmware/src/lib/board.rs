use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::Error as DError;
use serde::ser::Error as SError;
use core::result::Result;
use heapless::Vec;
use core::str;

use crate::layout::Layout;
use crate::consts::*;

type VMatrix<T> = Vec<Vec<T, MAX_COLS>, MAX_ROWS>;
type Matrix<T> = [[T; MAX_COLS]; MAX_ROWS];

#[derive(Debug, Serialize, Deserialize)]
pub struct Board {
  // matrix (i,j) -> register (idx, bit)
  pub matrix: VMatrix<(RegIndex,RegBitIndex)>,
  pub bus_pins: [PinIndex; BUS_WIDTH],
  pub switch_reg_pins: Vec<PinIndex, MAX_REGS>,
  pub backlight_reg_pins: Vec<PinIndex, MAX_REGS>,
  pub backlight_dim_pin: PinIndex,
  pub backlight_reset_pin: PinIndex,
  pub led_ind_pins: [PinIndex; 2],
}

pub struct RegMap {
  pub regs: Vec<[KeyIndex; BUS_WIDTH], MAX_REGS>,
}

pub fn make_reg_map(board: &Board, layout: &Layout) -> RegMap {
  let mut key_matrix: Matrix::<KeyIndex> = [[0; MAX_COLS]; MAX_ROWS];
  for idx in 0..layout.positions.len() {
    let (i,j) = layout.positions[idx];
    key_matrix[i][j] = idx as KeyIndex;
  }
  let mut reg_map: Vec<[KeyIndex; BUS_WIDTH], MAX_REGS> = Vec::new();
  let matrix = &board.matrix;
  for i in 0..matrix.len() {
    let row = &matrix[i];
    for j in 0..row.len() {
      let (reg_idx, reg_bit) = row[j];
      if reg_idx as usize >= reg_map.capacity() {
        // register higher than MAX_REGS, just skip for now
        continue;
      }
      if reg_idx as usize >= reg_map.len() {
        // will have capacity per previous check
        reg_map.resize_default((reg_idx+1) as usize).unwrap();
      }
      reg_map[reg_idx as usize][reg_bit as usize] = key_matrix[i][j];
    }
  }
  return RegMap { regs: reg_map };
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn can_deser_board() -> Result<(), String> {
    let (_board, _bytes_read): (Board, usize) =
      serde_json::from_slice(include_bytes!("../../boards/unchat-42.json"))
      .map_err(|e| format!("{}", e))?;
    Ok(())
  }
}
