use crate::composite_type::{Corners, FlipAxes};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DumbString(String);

impl DumbString {
  pub(crate) fn new(s: String) -> Self {
    Self(s)
  }

  pub(crate) fn to_string(&self) -> String {
    self.0.clone()
  }
}

impl From<String> for DumbString {
  fn from(value: String) -> Self {
    Self(value)
  }
}

pub(crate) struct TypeN<'a, T>(pub &'a [T]);

impl<'a, T> TypeN<'a, T> {
  pub(crate) fn new(arr: &'a [T]) -> Self {
    Self(arr)
  }
}

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

impl From<Corners> for OptionSet<u8> {
  fn from(c: Corners) -> Self {
    let b0: u8 = (if c.top_left { 1 } else { 0 }) << 0;
    let b1: u8 = (if c.top { 1 } else { 0 }) << 1;
    let b2: u8 = (if c.top_right { 1 } else { 0 }) << 2;
    let b3: u8 = (if c.right { 1 } else { 0 }) << 3;
    let b4: u8 = (if c.bottom_right { 1 } else { 0 }) << 4;
    let b5: u8 = (if c.bottom { 1 } else { 0 }) << 5;
    let b6: u8 = (if c.bottom_left { 1 } else { 0 }) << 6;
    let b7: u8 = (if c.left { 1 } else { 0 }) << 7;

    let v = b0 | b1 | b2 | b3 | b4 | b5 | b6 | b7;

    OptionSet(v)
  }
}

impl From<FlipAxes> for OptionSet<u8> {
  fn from(f: FlipAxes) -> Self {
    let b0: u8 = (if f.horizontal { 1 } else { 0 }) << 0;
    let b1: u8 = (if f.vertical { 1 } else { 0 }) << 1;

    let v = b0 | b1;

    OptionSet(v)
  }
}
