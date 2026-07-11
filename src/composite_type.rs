use crate::{error::ParseError, primitive_type::OptionSet};

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

impl TryFrom<&[u8]> for ArgbColor {
  type Error = ParseError<&'static [u8]>;

  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    let r = value
      .get(0)
      .ok_or_else(|| ParseError::InvalidArgbColorFormat)?;
    let g = value
      .get(1)
      .ok_or_else(|| ParseError::InvalidArgbColorFormat)?;
    let b = value
      .get(2)
      .ok_or_else(|| ParseError::InvalidArgbColorFormat)?;
    let a = value
      .get(3)
      .ok_or_else(|| ParseError::InvalidArgbColorFormat)?;

    Ok(Self {
      r: *r,
      g: *g,
      b: *b,
      a: *a,
    })
  }
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
  type Error = ParseError<&'static [u8]>;

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
      _ => Err(ParseError::InvalidBlendMode),
    }
  }
}

/// Type of custom data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomDataType {
  String,
}

impl TryFrom<u8> for CustomDataType {
  type Error = ParseError<&'static [u8]>;

  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(Self::String),
      _ => Err(ParseError::InvalidCustomDataType),
    }
  }
}

/// Type of Fx.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FxType {
  ColorOverlay,
  Outline,
  AntiAliasing,
  PatternOverlay,
}

impl TryFrom<u8> for FxType {
  type Error = ParseError<&'static [u8]>;

  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(Self::ColorOverlay),
      1 => Ok(Self::Outline),
      3 => Ok(Self::AntiAliasing),
      4 => Ok(Self::PatternOverlay),
      _ => Err(ParseError::InvalidFxType),
    }
  }
}

/// Type of Entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
  RegularLayer,
  Group,
  ReferenceLayer,
  TilemapLayer,
}

impl TryFrom<u8> for EntryType {
  type Error = ParseError<&'static [u8]>;

  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(Self::RegularLayer),
      1 => Ok(Self::Group),
      2 => Ok(Self::ReferenceLayer),
      3 => Ok(Self::TilemapLayer),
      _ => Err(ParseError::InvalidEntryType),
    }
  }
}

/// Type of GuideLine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideLineType {
  Grid,
  Isometric,
  Perspective,
}

impl TryFrom<u8> for GuideLineType {
  type Error = ParseError<&'static [u8]>;

  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(Self::Grid),
      1 => Ok(Self::Isometric),
      2 => Ok(Self::Perspective),
      _ => Err(ParseError::InvalidGuideLineType),
    }
  }
}

/// Type of GuideLine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorType {
  Crt,
  Vignette,
  Bloom,
  RoundPixel,
}

impl TryFrom<u8> for ProcessorType {
  type Error = ParseError<&'static [u8]>;

  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(Self::Crt),
      1 => Ok(Self::Vignette),
      2 => Ok(Self::Bloom),
      3 => Ok(Self::RoundPixel),
      _ => Err(ParseError::InvalidProcessorType),
    }
  }
}

/// Type of Modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierType {
  AnimationSpeedMultiplier,
}

impl TryFrom<u8> for ModifierType {
  type Error = ParseError<&'static [u8]>;

  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(Self::AnimationSpeedMultiplier),
      _ => Err(ParseError::InvalidModifierType),
    }
  }
}

/// Type of PaletteOrganization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteOrganizationType {
  Packed,
  Anywhere,
}

impl TryFrom<u8> for PaletteOrganizationType {
  type Error = ParseError<&'static [u8]>;

  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(Self::Packed),
      1 => Ok(Self::Anywhere),
      _ => Err(ParseError::InvalidPaletteOrganizationType),
    }
  }
}

/// Flip Axes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlipAxes {
  /// true if flipped horizontally.
  pub horizontal: bool,
  /// true if flipped vertically.
  pub vertical: bool,
}

/// Symmetry type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetryType {
  Mirror,
  Rotate,
}

impl From<u8> for SymmetryType {
  fn from(value: u8) -> Self {
    if value == 0 {
      return Self::Mirror;
    }

    Self::Rotate
  }
}

/// A direction of frame animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationDirection {
  Forward,
  Backward,
  PingPong,
}

impl TryFrom<u8> for AnimationDirection {
  type Error = ParseError<&'static [u8]>;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(Self::Forward),
      1 => Ok(Self::Backward),
      2 => Ok(Self::PingPong),
      _ => Err(ParseError::InvalidAnimationDirection),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDepth {
  Rgb,
  Indexed,
}

impl TryFrom<u8> for ColorDepth {
  type Error = ParseError<&'static [u8]>;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(Self::Rgb),
      1 => Ok(Self::Indexed),
      _ => Err(ParseError::InvalidColorDepth),
    }
  }
}
