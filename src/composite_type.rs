use crate::primitive_type::OptionSet;

/// https://docs.pixquare.art/pixquare-file/binary-specs#coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinate {
  pub x: i32,
  pub y: i32,
}

/// https://docs.pixquare.art/pixquare-file/binary-specs#corners
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Corners {
  pub top_left: bool,
  pub top: bool,
  pub top_right: bool,
  pub right: bool,
  pub bottom_right: bool,
  pub bottom: bool,
  pub bottom_left: bool,
  pub left: bool,
}

impl From<OptionSet<u8>> for Corners {
  fn from(os: OptionSet<u8>) -> Self {
    Self {
      top_left: os.flag(0),
      top: os.flag(1),
      top_right: os.flag(2),
      right: os.flag(3),
      bottom_right: os.flag(4),
      bottom: os.flag(5),
      bottom_left: os.flag(6),
      left: os.flag(7),
    }
  }
}
