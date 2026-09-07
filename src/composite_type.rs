use std::io::Write;

use msgw3c::Blend;

use crate::{
  error::{PMResult, ParseError},
  marshaler::Marshal,
  primitive_type::OptionSet,
};

/// <https://docs.pixquare.art/pixquare-file/binary-specs#coordinate>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinate {
  pub x: i32,
  pub y: i32,
}

impl Marshal for Coordinate {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    self.x.marshal(w)?;
    self.y.marshal(w)?;

    Ok(())
  }
}

/// <https://docs.pixquare.art/pixquare-file/binary-specs#size>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
  pub width: u32,
  pub height: u32,
}

impl Size {
  pub fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }
}

impl Marshal for Size {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    self.width.marshal(w)?;
    self.height.marshal(w)?;

    Ok(())
  }
}

/// <https://docs.pixquare.art/pixquare-file/binary-specs#rect>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
  pub origin: Coordinate,
  pub size: Size,
}

impl Marshal for Rect {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    self.origin.marshal(w)?;
    self.size.marshal(w)?;

    Ok(())
  }
}

/// <https://docs.pixquare.art/pixquare-file/binary-specs#argbcolor>
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ArgbColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
}

impl ArgbColor {
  pub const TRANSPARENT: Self = Self {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
  };

  pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
    Self { r, g, b, a }
  }

  pub fn from_straight_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
    let alpha_ratio = (a as f32) / 255.;

    Self {
      r: ((r as f32) * alpha_ratio) as u8,
      g: ((g as f32) * alpha_ratio) as u8,
      b: ((b as f32) * alpha_ratio) as u8,
      a,
    }
  }
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

impl Blend for ArgbColor {
  fn from_color(c: msgw3c::color::C) -> Self {
    Self::new(
      (c.r * 255.).clamp(0.0, 255.0) as u8,
      (c.g * 255.).clamp(0.0, 255.0) as u8,
      (c.b * 255.).clamp(0.0, 255.0) as u8,
      (c.a * 255.).clamp(0.0, 255.0) as u8,
    )
  }

  fn to_color(&self) -> msgw3c::color::C {
    msgw3c::color::C {
      r: self.r as f32 / 255.,
      g: self.g as f32 / 255.,
      b: self.b as f32 / 255.,
      a: self.a as f32 / 255.,
    }
  }
}

impl Marshal for ArgbColor {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    self.r.marshal(w)?;
    self.g.marshal(w)?;
    self.b.marshal(w)?;
    self.a.marshal(w)?;

    Ok(())
  }
}

/// <https://docs.pixquare.art/pixquare-file/binary-specs#corners>
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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
      top_left: os.n_flag(0),
      top: os.n_flag(1),
      top_right: os.n_flag(2),
      right: os.n_flag(3),
      bottom_right: os.n_flag(4),
      bottom: os.n_flag(5),
      bottom_left: os.n_flag(6),
      left: os.n_flag(7),
    }
  }
}

impl Marshal for Corners {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let o: OptionSet<u8> = (*self).into();

    o.marshal(w)?;

    Ok(())
  }
}

/// <https://docs.pixquare.art/pixquare-file/binary-specs#blendmode>
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

impl From<BlendMode> for u16 {
  fn from(value: BlendMode) -> Self {
    match value {
      BlendMode::Normal => 0,
      BlendMode::Multiply => 1,
      BlendMode::Screen => 2,
      BlendMode::Overlay => 3,
      BlendMode::Darken => 4,
      BlendMode::Lighten => 5,
      BlendMode::ColorDodge => 6,
      BlendMode::ColorBurn => 7,
      BlendMode::HardLight => 8,
      BlendMode::SoftLight => 9,
      BlendMode::Difference => 10,
      BlendMode::Exclusion => 11,
      BlendMode::Hue => 12,
      BlendMode::Saturation => 13,
      BlendMode::Color => 14,
      BlendMode::Luminosity => 15,
    }
  }
}

impl Marshal for BlendMode {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: u16 = (*self).into();

    w.write_all(&v.to_le_bytes())?;

    Ok(())
  }
}

impl From<BlendMode> for msgw3c::blend::BlendMode {
  fn from(value: BlendMode) -> Self {
    match value {
      BlendMode::Normal => msgw3c::blend::BlendMode::Normal,
      BlendMode::Multiply => msgw3c::blend::BlendMode::Multiply,
      BlendMode::Screen => msgw3c::blend::BlendMode::Screen,
      BlendMode::Overlay => msgw3c::blend::BlendMode::Overlay,
      BlendMode::Darken => msgw3c::blend::BlendMode::Darken,
      BlendMode::Lighten => msgw3c::blend::BlendMode::Lighten,
      BlendMode::ColorDodge => msgw3c::blend::BlendMode::ColorDodge,
      BlendMode::ColorBurn => msgw3c::blend::BlendMode::ColorBurn,
      BlendMode::HardLight => msgw3c::blend::BlendMode::HardLight,
      BlendMode::SoftLight => msgw3c::blend::BlendMode::SoftLight,
      BlendMode::Difference => msgw3c::blend::BlendMode::Difference,
      BlendMode::Exclusion => msgw3c::blend::BlendMode::Exclusion,
      BlendMode::Hue => msgw3c::blend::BlendMode::Hue,
      BlendMode::Saturation => msgw3c::blend::BlendMode::Saturation,
      BlendMode::Color => msgw3c::blend::BlendMode::Color,
      BlendMode::Luminosity => msgw3c::blend::BlendMode::Luminosity,
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

impl From<CustomDataType> for u8 {
  fn from(value: CustomDataType) -> Self {
    match value {
      CustomDataType::String => 0,
    }
  }
}

impl Marshal for CustomDataType {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: u8 = (*self).into();

    w.write_all(&v.to_le_bytes())?;

    Ok(())
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
      2 => Ok(Self::AntiAliasing),
      3 => Ok(Self::PatternOverlay),
      _ => Err(ParseError::InvalidFxType),
    }
  }
}

impl From<FxType> for u8 {
  fn from(value: FxType) -> Self {
    match value {
      FxType::ColorOverlay => 0,
      FxType::Outline => 1,
      FxType::AntiAliasing => 2,
      FxType::PatternOverlay => 3,
    }
  }
}

impl Marshal for FxType {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: u8 = (*self).into();

    w.write_all(&v.to_le_bytes())?;

    Ok(())
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

impl From<EntryType> for u8 {
  fn from(value: EntryType) -> Self {
    match value {
      EntryType::RegularLayer => 0,
      EntryType::Group => 1,
      EntryType::ReferenceLayer => 2,
      EntryType::TilemapLayer => 3,
    }
  }
}

impl Marshal for EntryType {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: u8 = (*self).into();

    w.write_all(&v.to_le_bytes())?;

    Ok(())
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

impl From<GuideLineType> for u8 {
  fn from(value: GuideLineType) -> Self {
    match value {
      GuideLineType::Grid => 0,
      GuideLineType::Isometric => 1,
      GuideLineType::Perspective => 2,
    }
  }
}

impl Marshal for GuideLineType {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: u8 = (*self).into();

    w.write_all(&v.to_le_bytes())?;

    Ok(())
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

impl From<ProcessorType> for u8 {
  fn from(value: ProcessorType) -> Self {
    match value {
      ProcessorType::Crt => 0,
      ProcessorType::Vignette => 1,
      ProcessorType::Bloom => 2,
      ProcessorType::RoundPixel => 3,
    }
  }
}

impl Marshal for ProcessorType {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: u8 = (*self).into();

    w.write_all(&v.to_le_bytes())?;

    Ok(())
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

impl From<ModifierType> for u8 {
  fn from(value: ModifierType) -> Self {
    match value {
      ModifierType::AnimationSpeedMultiplier => 0,
    }
  }
}

impl Marshal for ModifierType {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: u8 = (*self).into();

    w.write_all(&v.to_le_bytes())?;

    Ok(())
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

impl From<PaletteOrganizationType> for u8 {
  fn from(value: PaletteOrganizationType) -> Self {
    match value {
      PaletteOrganizationType::Packed => 0,
      PaletteOrganizationType::Anywhere => 1,
    }
  }
}

impl Marshal for PaletteOrganizationType {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: u8 = (*self).into();

    w.write_all(&v.to_le_bytes())?;

    Ok(())
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

impl Marshal for FlipAxes {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: OptionSet<u8> = (*self).into();

    v.marshal(w)?;

    Ok(())
  }
}

/// Symmetry type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymmetryType {
  Mirror,
  Rotate,
}

impl TryFrom<u8> for SymmetryType {
  type Error = ParseError<&'static [u8]>;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(Self::Mirror),
      1 => Ok(Self::Rotate),
      _ => Err(ParseError::InvalidSymmetryLine),
    }
  }
}

impl From<SymmetryType> for u8 {
  fn from(value: SymmetryType) -> Self {
    match value {
      SymmetryType::Mirror => 0,
      SymmetryType::Rotate => 1,
    }
  }
}

impl Marshal for SymmetryType {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: u8 = (*self).into();
    v.marshal(w)?;

    Ok(())
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

impl From<AnimationDirection> for u8 {
  fn from(value: AnimationDirection) -> Self {
    match value {
      AnimationDirection::Forward => 0,
      AnimationDirection::Backward => 1,
      AnimationDirection::PingPong => 2,
    }
  }
}

impl Marshal for AnimationDirection {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: u8 = (*self).into();
    v.marshal(w)?;

    Ok(())
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

impl From<ColorDepth> for u8 {
  fn from(value: ColorDepth) -> Self {
    match value {
      ColorDepth::Rgb => 0,
      ColorDepth::Indexed => 1,
    }
  }
}

impl Marshal for ColorDepth {
  fn marshal<W: Write>(&self, w: &mut W) -> PMResult<()> {
    let v: u8 = (*self).into();
    v.marshal(w)?;

    Ok(())
  }
}
