use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::Error as DError;
use serde::ser::Error as SError;
use core::result::Result;
use heapless::Vec;
use core::str;

use ehal::digital::v2::{InputPin, OutputPin};

use crate::layout::Layout;
use crate::bus::TryIntoOutputPin;
use crate::prelude::*;

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

pub struct BoardPins<P: InputPin, Q: OutputPin> {
  pub bus_pins: [P; BUS_WIDTH],
  pub switch_reg_pins: Vec<Q, MAX_REGS>,
  pub backlight_reg_pins: Vec<Q, MAX_REGS>,
  pub backlight_dim_pin: Q,
  pub backlight_reset_pin: Q,
  pub led_ind_pins: [Q; 2],
}
pub fn split_pins<P: InputPin, Q: OutputPin, const N: usize>(
  mut pins: Vec<P, N>, mut ids: Vec<usize, N>, board: &Board
) -> Result<BoardPins<P, Q>, Error>
where P: TryIntoOutputPin<Pin=Q>
{
  let bus_pins: [P; BUS_WIDTH] =
    board.bus_pins.iter().map(|p| {
      let idx = ids.iter().position(|&i| i == *p as usize)
        .ok_or(Error::PinConfigError)?;
      ids.swap_remove(idx);
      Ok(pins.swap_remove(idx))
    }).collect::<Result<Vec<P, BUS_WIDTH>, Error>>()?
    .into_array().map_err(|_| Error::PinConfigError)?;
  let switch_reg_pins: Vec<Q, MAX_REGS> =
    board.switch_reg_pins.iter().map(|p| {
      let idx = ids.iter().position(|&i| i == *p as usize)
        .ok_or(Error::PinConfigError)?;
      ids.swap_remove(idx);
      Ok(pins.swap_remove(idx).try_into_output_pin()?)
    }).collect::<Result<Vec<Q, MAX_REGS>, Error>>()?;
  let backlight_reg_pins: Vec<Q, MAX_REGS> =
    board.backlight_reg_pins.iter().map(|p| {
      let idx = ids.iter().position(|&i| i == *p as usize)
        .ok_or(Error::PinConfigError)?;
      ids.swap_remove(idx);
      Ok(pins.swap_remove(idx).try_into_output_pin()?)
    }).collect::<Result<Vec<Q, MAX_REGS>, Error>>()?;
  let backlight_dim_pin = {
    let p = &board.backlight_dim_pin;
    let idx = ids.iter().position(|&i| i == *p as usize)
      .ok_or(Error::PinConfigError)?;
    ids.swap_remove(idx);
    Ok(pins.swap_remove(idx).try_into_output_pin()?)
  }?;
  let backlight_reset_pin = {
    let p = &board.backlight_reset_pin;
    let idx = ids.iter().position(|&i| i == *p as usize)
      .ok_or(Error::PinConfigError)?;
    ids.swap_remove(idx);
    Ok(pins.swap_remove(idx).try_into_output_pin()?)
  }?;
  let led_ind_pins: [Q; 2] =
    board.led_ind_pins.iter().map(|p| {
      let idx = ids.iter().position(|&i| i == *p as usize)
        .ok_or(Error::PinConfigError)?;
      ids.swap_remove(idx);
      Ok(pins.swap_remove(idx).try_into_output_pin()?)
    }).collect::<Result<Vec<Q, 2>, Error>>()?
    .into_array().map_err(|_| Error::PinConfigError)?;
  Ok(BoardPins {
    bus_pins, switch_reg_pins, backlight_reg_pins, backlight_dim_pin,
    backlight_reset_pin, led_ind_pins,
  })
}

#[derive(Clone)]
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
