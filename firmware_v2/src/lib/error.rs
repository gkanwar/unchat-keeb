#[derive(Debug, Copy, Clone)]
pub enum Error {
  PinConfigError,
  UsbError,
  VecOverflow,
  SizeMismatch,
}
