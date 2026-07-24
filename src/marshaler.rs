use std::io::Write;

use half::f16;

use crate::{
  error::PMResult,
  primitive_type::{DumbString, OptionSet, TypeN},
};

/// Marshal provides a function that converts the implemented type to a binary and writes it to a Writer.
pub(crate) trait Marshal<T = ()> {
  /// Marshal to binary and write it to the writer.
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<T>;
}

impl Marshal for u8 {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    w.write_all(&self.to_le_bytes())?;

    Ok(())
  }
}

impl Marshal for u16 {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    w.write_all(&self.to_le_bytes())?;

    Ok(())
  }
}

impl Marshal for u32 {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    w.write_all(&self.to_le_bytes())?;

    Ok(())
  }
}

impl Marshal for u64 {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    w.write_all(&self.to_le_bytes())?;

    Ok(())
  }
}

impl Marshal for i32 {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    w.write_all(&self.to_le_bytes())?;

    Ok(())
  }
}

impl Marshal for f16 {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    w.write_all(&self.to_le_bytes())?;

    Ok(())
  }
}

impl Marshal for f32 {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    w.write_all(&self.to_le_bytes())?;

    Ok(())
  }
}

impl Marshal for f64 {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    w.write_all(&self.to_le_bytes())?;

    Ok(())
  }
}

impl Marshal for bool {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let b: u8 = if *self { 1 } else { 0 };
    w.write_all(&b.to_le_bytes())?;

    Ok(())
  }
}

impl Marshal for OptionSet<u8> {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let b0 = self.n_bit(0);
    let b1 = self.n_bit(1);
    let b2 = self.n_bit(2);
    let b3 = self.n_bit(3);
    let b4 = self.n_bit(4);
    let b5 = self.n_bit(5);
    let b6 = self.n_bit(6);
    let b7 = self.n_bit(7);

    let data = b0 | b1 | b2 | b3 | b4 | b5 | b6 | b7;
    w.write_all(&data.to_le_bytes())?;

    Ok(())
  }
}

impl Marshal<usize> for String {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<usize> {
    // write string length
    let s = self.to_string();
    let len = s.len();
    w.write_all(&(len as u16).to_le_bytes())?;

    w.write_all(s.as_bytes())?;

    Ok(len)
  }
}

impl Marshal for DumbString {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    w.write_all(self.to_string().as_bytes())?;

    Ok(())
  }
}

impl<T> Marshal for Vec<T>
where
  T: Marshal,
{
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let len = self.len() as u64;
    len.marshal(w)?;

    for i in 0..len {
      self[i as usize].marshal(w)?;
    }

    Ok(())
  }
}

impl<T, const N: usize> Marshal for [T; N]
where
  T: Marshal,
{
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let len = self.len() as u64;

    for i in 0..len {
      self[i as usize].marshal(w)?;
    }

    Ok(())
  }
}

impl<'a, T> Marshal for TypeN<'a, T>
where
  T: Marshal,
{
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    for i in 0..self.0.len() {
      self.0[i].marshal(w)?;
    }

    Ok(())
  }
}
