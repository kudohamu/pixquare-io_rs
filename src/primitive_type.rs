pub trait Uint {}

impl Uint for u8 {}

/// A set of true (1) - false (0) flags using each bit of the UInt type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionSet<U: Uint>(U);

impl<U: Uint> OptionSet<U> {
  pub fn new(v: U) -> Self {
    Self(v)
  }
}

impl OptionSet<u8> {
  pub fn flag(&self, pos: u8) -> bool {
    (self.0 & (1 << pos)) != 0
  }
}
