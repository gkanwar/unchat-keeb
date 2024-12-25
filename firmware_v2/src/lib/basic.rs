use core::fmt;
use heapless::Vec;


pub struct WriteBuf<const N: usize> {
  pub data: Vec<u8, N>
}

impl<const N: usize> WriteBuf<N> {
  pub fn new() -> Self {
    Self { data: Vec::new() }
  }
}

impl<const N: usize> fmt::Write for WriteBuf<N> {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    let n = s.len().min(self.data.capacity() - self.data.len());
    self.data.extend_from_slice(s[..n].as_bytes()).map_err(|_| fmt::Error::default())
  }
}
