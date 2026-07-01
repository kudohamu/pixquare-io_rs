/// A set of true (1) - false (0) flags using each bit of the uint type.
pub trait OptionSet {
  fn flag(&self, pos: usize) -> bool;
}

/// A set of true (1) - false (0) flags using each bit of the u8 type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionSetU8(u8);

impl OptionSetU8 {
  pub fn new(v: u8) -> Self {
    Self(v)
  }
}

impl OptionSet for OptionSetU8 {
  fn flag(&self, pos: usize) -> bool {
    (self.0 & (1 << pos)) != 0
  }
}
