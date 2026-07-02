use crate::{error::Error, primitive_type::OptionSet};

/// https://docs.pixquare.art/pixquare-file/binary-specs#coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinate {
  pub x: i32,
  pub y: i32,
}

/// https://docs.pixquare.art/pixquare-file/binary-specs#size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
  pub width: u32,
  pub height: u32,
}

/// https://docs.pixquare.art/pixquare-file/binary-specs#rect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
  pub origin: Coordinate,
  pub size: Size,
}

/// https://docs.pixquare.art/pixquare-file/binary-specs#argbcolor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgbColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
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

/// https://docs.pixquare.art/pixquare-file/binary-specs#blendmode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
  Normal,
  Multiply,
  Screen,
  Overlay,
  Darken,
  Lighten,
  ColorDodge,
  ColorBurn,
  HardLight,
  SoftLight,
  Difference,
  Exclusion,
  Hue,
  Saturation,
  Color,
  Luminosity,
}

impl TryFrom<u16> for BlendMode {
  type Error = Error;

  fn try_from(v: u16) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(Self::Normal),
      1 => Ok(Self::Multiply),
      2 => Ok(Self::Screen),
      3 => Ok(Self::Overlay),
      4 => Ok(Self::Darken),
      5 => Ok(Self::Lighten),
      6 => Ok(Self::ColorDodge),
      7 => Ok(Self::ColorBurn),
      8 => Ok(Self::HardLight),
      9 => Ok(Self::SoftLight),
      10 => Ok(Self::Difference),
      11 => Ok(Self::Exclusion),
      12 => Ok(Self::Hue),
      13 => Ok(Self::Saturation),
      14 => Ok(Self::Color),
      15 => Ok(Self::Luminosity),
      _ => Err(Error::InvalidBlendMode),
    }
  }
}
