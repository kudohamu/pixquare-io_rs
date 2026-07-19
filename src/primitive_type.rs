#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DumbString(String);

impl DumbString {
  pub fn to_string(&self) -> String {
    self.0.clone()
  }
}

impl From<String> for DumbString {
  fn from(value: String) -> Self {
    Self(value)
  }
}

pub(crate) struct TypeN<T>(pub Vec<T>);

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
  // Returns whether the n-index position is true or false.
  pub fn n_flag(&self, pos: u8) -> bool {
    self.n_bit(pos) != 0
  }

  // Returns the bit in the n-index position as u8.
  pub fn n_bit(&self, pos: u8) -> u8 {
    self.0 & (1 << pos)
  }
}
