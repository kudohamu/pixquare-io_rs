mod combinator;
mod composite_type;
mod primitive_type;

/// The pixquare(.px) file data.
#[derive(Debug)]
pub struct PixquareFile {}

impl PixquareFile {
  /// Load a .px file from a byte slice.
  pub fn load(_file_data: &[u8]) -> Result<Self, ()> {
    Ok(Self {})
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn load_file_from_slice() {
    let path = "assets/fixtures/simple.px";
    let file_data = std::fs::read(path).unwrap();
    let file = PixquareFile::load(&file_data);

    assert!(file.is_ok());
  }
}
