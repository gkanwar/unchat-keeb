#[derive(Debug, Copy, Clone)]
pub enum Error {
  PinConfigError,
  VecOverflow,
  SizeMismatch,
}
