use std::io::Read;

use flate2::read::ZlibDecoder;
use half::f16;
use nom::{
  Parser,
  bytes::complete::take,
  combinator::{map, map_res},
  error::ErrorKind,
  multi::many0,
  number::complete::{le_f32, le_f64, le_u8, le_u16, le_u32, le_u64},
};

use crate::{
  combinator::{
    argb_color, array_type, blend_mode, bool, compressed_colors, corners, dumb_string, float16,
    option_set_u8, rect, size, string, type_n,
  },
  composite_type::{
    AnimationDirection, ArgbColor, BlendMode, ColorDepth, Corners, CustomDataType, EntryType,
    FlipAxes, FxType, GuideLineType, ModifierType, PaletteOrganizationType, ProcessorType, Rect,
    Size, SymmetryType,
  },
  error::{PQResult, ParseError},
  primitive_type::OptionSet,
};

/// Header data of CustomData.
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-16-bytes
#[derive(Debug)]
struct CustomDataHeader {
  /// Total size of this model.
  data_size: u64,
  /// Data type.
  data_type: CustomDataType,
}

impl CustomDataHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(16usize).parse(input)?;

    let (_input, (data_size, data_type)) =
      (le_u64, le_u8.map_res(|b| b.try_into())).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        data_type,
      },
    ))
  }
}

/// Content data of string for CustomData.
#[derive(Debug, Clone)]
pub struct CustomDataStringContent {
  pub content: String,
  remaining_data: Vec<u8>,
}

impl<'a> CustomDataStringContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (remaining_data, str) = string(input)?;

      Ok((
        rest,
        Self {
          content: str.to_string(),
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// Store some custom data set by the users.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content
#[derive(Debug, Clone)]
pub enum CustomData {
  String(CustomDataStringContent),
}

impl CustomData {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = CustomDataHeader::parse(input)?;

    match header.data_type {
      CustomDataType::String => {
        let (input, content) =
          CustomDataStringContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Self::String(content)))
      }
    }
  }
}

/// Header data of FrameContent.
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-32-bytes
#[derive(Debug)]
struct FrameContentHeader {
  /// Size of this model.
  data_size: u64,
  /// ID length.
  id_len: u8,
  /// Uncompressed color data length.
  /// width x height x pixel length (4 for RGBA and 1 for indexed).
  color_len: u32,
  /// Color data length.
  compressed_color_len: u32,
  /// Backward compatibility. Always `0b00000001` .
  _compat: OptionSet<u8>,
}

impl FrameContentHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(32usize).parse(input)?;

    let (_input, (data_size, id_len, color_len, compressed_color_len, _compat)) =
      (le_u64, le_u8, le_u32, le_u32, option_set_u8).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        id_len,
        color_len,
        compressed_color_len,
        _compat,
      },
    ))
  }
}

/// The color data of a cel.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-1
#[derive(Debug, Clone)]
pub struct FrameContent {
  pub id: String,
  pub colors: Vec<ArgbColor>,
  remaining_data: Vec<u8>,
}

impl FrameContent {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = FrameContentHeader::parse(input)?;
    let (rest, input) = take(header.data_size as usize).parse(input)?;

    let (input, id) = dumb_string(header.id_len as usize).parse(input)?;
    let (remaining_data, colors) =
      compressed_colors(header.compressed_color_len as usize).parse(input)?;

    Ok((
      rest,
      Self {
        id: id.to_string(),
        colors,
        remaining_data: remaining_data.into(),
      },
    ))
  }
}

/// Header data of TilemapFrameContent.
/// 32 bytes
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-32-bytes-1
#[derive(Debug)]
struct TilemapFrameContentHeader {
  /// Size of this model.
  data_size: u64,
}

impl TilemapFrameContentHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(32usize).parse(input)?;

    let (_input, data_size) = le_u64.parse(input)?;

    Ok((rest, Self { data_size }))
  }
}

/// The alignment of a tilemap cel.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-2
#[derive(Debug, Clone)]
pub struct TilemapFrameContent {
  pub id: String,
  pub tile_size: Size,
  /// Alignment of tile on the canvas.
  /// UInt16.max is for unassigned tiles.
  pub tiles: Vec<u16>,
  remaining_data: Vec<u8>,
}

impl TilemapFrameContent {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = TilemapFrameContentHeader::parse(input)?;
    let (rest, input) = take(header.data_size as usize).parse(input)?;

    let (input, (id, tile_size, tiles_len)) = (string, size, le_u64).parse(input)?;
    // this prefix is a byte length, not array length.
    // `2` is byte size of each element.
    let (remaining_data, tiles) = type_n(le_u16, (tiles_len / 2) as usize).parse(input)?;

    Ok((
      rest,
      Self {
        id: id.to_string(),
        tile_size,
        tiles,
        remaining_data: remaining_data.into(),
      },
    ))
  }
}

/// Header data of Frame.
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-32-bytes-2
#[derive(Debug)]
struct FrameHeader {
  /// Size of this model.
  data_size: u32,
  /// ID length.
  id_len: u8,
  /// Content ID length.
  content_len: u8,
}

impl FrameHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(32usize).parse(input)?;

    let (_input, (data_size, id_len, content_len)) = (le_u32, le_u8, le_u8).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        id_len,
        content_len,
      },
    ))
  }
}

/// A frame of the artwork.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-3
#[derive(Debug, Clone)]
pub struct Frame {
  /// ID.
  /// The length is specified in the header.
  /// Frames with the same index have the same ID.
  pub id: String,
  /// Duration.
  /// Frames with the same index have the same duration.
  pub duration: u32,
  pub selected: bool,
  /// Content ID
  /// The length is specified in the header.
  /// This is the ID of the `FrameContent` for `Layer` and `TilemapFrameContent` for `TilemapLayer`.
  pub content_id: String,
  /// Value from 0 to 1 is the actual value.
  /// 2 means using the opacity of the Layer.
  /// Default: 2
  pub opacity: f16,
  /// Default: 0.
  pub z_index: u16,
  /// Custom datas set by the users.
  pub custom_datas: Vec<CustomData>,
  remaining_data: Vec<u8>,
}

impl Frame {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = FrameHeader::parse(input)?;
    let (rest, input) = take(header.data_size as usize).parse(input)?;

    let (remaining_data, (id, duration, selected, content_id, opacity, z_index, custom_datas)) = (
      dumb_string(header.id_len as usize),
      le_u32,
      bool,
      dumb_string(header.content_len as usize),
      float16,
      le_u16,
      array_type(CustomData::parse),
    )
      .parse(input)?;

    Ok((
      rest,
      Self {
        id: id.to_string(),
        duration,
        selected,
        content_id: content_id.to_string(),
        opacity,
        z_index,
        custom_datas,
        remaining_data: remaining_data.into(),
      },
    ))
  }
}

/// Header data of SymmetryLine.
/// 16 bytes.
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-16-bytes-1
#[derive(Debug)]
struct SymmetryLineHeader {
  /// Size of this model.
  data_size: u32,
  /// ID length.
  id_len: u8,
  /// Backward compatibility. Always `0b00000011` .
  _compat: OptionSet<u8>,
}

impl SymmetryLineHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(16usize).parse(input)?;

    let (_input, (data_size, id_len, _compat)) = (le_u32, le_u8, option_set_u8).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        id_len,
        _compat,
      },
    ))
  }
}

/// Settings of symmetry line.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-4
#[derive(Debug, Clone)]
pub struct SymmetryLine {
  /// ID.
  /// The length is specified in the header.
  pub id: String,
  pub color: ArgbColor,
  pub selected: bool,
  pub enabled: bool,
  /// x-value of the origin.
  pub origin_x: f32,
  /// y-value of the origin.
  pub origin_y: f32,
  /// Rotation angle in radians.
  pub angle: f32,
  pub segment_count: u8,
  pub symmetry_type: SymmetryType,
  remaining_data: Vec<u8>,
}

impl SymmetryLine {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = SymmetryLineHeader::parse(input)?;
    let (rest, input) = take(header.data_size as usize).parse(input)?;

    let (
      remaining_data,
      (id, color, selected, enabled, origin_x, origin_y, angle, segment_count, symmetry_type),
    ) = (
      dumb_string(header.id_len as usize),
      argb_color,
      bool,
      bool,
      le_f32,
      le_f32,
      le_f32,
      le_u8,
      map(le_u8, |b| b.into()),
    )
      .parse(input)?;

    Ok((
      rest,
      Self {
        id: id.to_string(),
        color,
        selected,
        enabled,
        origin_x,
        origin_y,
        angle,
        segment_count,
        symmetry_type,
        remaining_data: remaining_data.into(),
      },
    ))
  }
}

/// Header data of Tag.
/// 16 bytes.
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-16-bytes-2
#[derive(Debug)]
struct TagHeader {
  /// Size of this model.
  data_size: u32,
  /// ID length.
  id_len: u8,
  /// Name length.
  name_len: u8,
}

impl TagHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(16usize).parse(input)?;

    let (_input, (data_size, id_len, name_len)) = (le_u32, le_u8, le_u8).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        id_len,
        name_len,
      },
    ))
  }
}

/// Tag is a collection data of consecutive frames.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-5
#[derive(Debug, Clone)]
pub struct Tag {
  /// ID.
  /// The length is specified in the header.
  pub id: String,
  /// Name.
  /// The length is specified in the header.
  pub name: String,
  /// Start frame index.
  pub start_index: u16,
  /// End frame index.
  pub end_index: u16,
  pub selected: bool,
  pub color: ArgbColor,
  pub direction: AnimationDirection,
  pub loop_count: u16,
  remaining_data: Vec<u8>,
}

impl Tag {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = TagHeader::parse(input)?;
    let (rest, input) = take(header.data_size as usize).parse(input)?;

    let (
      remaining_data,
      (id, name, start_index, end_index, selected, color, direction, loop_count),
    ) = (
      dumb_string(header.id_len as usize),
      dumb_string(header.name_len as usize),
      le_u16,
      le_u16,
      bool,
      argb_color,
      map_res(le_u8, |b| b.try_into()),
      le_u16,
    )
      .parse(input)?;

    Ok((
      rest,
      Self {
        id: id.to_string(),
        name: name.to_string(),
        start_index,
        end_index,
        selected,
        color,
        direction,
        loop_count,
        remaining_data: remaining_data.into(),
      },
    ))
  }
}

/// Header data of Tileset.
/// 32 bytes
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-32-bytes-3
#[derive(Debug)]
struct TilesetHeader {
  /// Size of this model.
  data_size: u32,
}

impl TilesetHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(32usize).parse(input)?;

    let (_input, data_size) = le_u32.parse(input)?;

    Ok((rest, Self { data_size }))
  }
}

/// 32 bytes
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-6
#[derive(Debug, Clone)]
pub struct Tileset {
  pub id: String,
  pub name: String,
  pub tile_size: Size,
  /// Compressed color data of each tile using zlib compression.
  /// After decompressing, it will be in the form of [ARGBColor].
  pub tile_images: Vec<Vec<ArgbColor>>,
  /// Tiles per row.
  /// Default: 6
  pub tiles_per_row: u16,
  /// Default: Black
  pub grid_color: ArgbColor,
  remaining_data: Vec<u8>,
}

impl Tileset {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = TilesetHeader::parse(input)?;
    let (rest, input) = take(header.data_size as usize).parse(input)?;

    let (remaining_data, (id, name, tile_size, tile_images, tiles_per_row, grid_color)) = (
      string,
      string,
      size,
      array_type(Tileset::compressed_colors),
      le_u16,
      argb_color,
    )
      .parse(input)?;

    Ok((
      rest,
      Self {
        id: id.to_string(),
        name: name.to_string(),
        tile_size,
        tile_images,
        tiles_per_row,
        grid_color,
        remaining_data: remaining_data.into(),
      },
    ))
  }

  /// Combinator of compressed argb colors for tile images.
  /// The Tileset documentation labels this as `[ARGBColor]`,
  /// but each compressed tile contains the raw, consecutive ARGBColor values.
  /// Unlike other `[Type]` values in the format, there is no leading UInt64 count.
  fn compressed_colors(input: &[u8]) -> PQResult<&[u8], Vec<ArgbColor>> {
    let (input, compressed_len) = le_u64.parse(input)?;
    let (input, compressed_data) = take(compressed_len as usize)(input)?;

    let mut decoder = ZlibDecoder::new(compressed_data);
    let mut decompressed_bytes = Vec::new();
    decoder
      .read_to_end(&mut decompressed_bytes)
      .map_err(|_e| nom::Err::Error(ParseError::DecompressZlibError))?;

    let (_remaining_decompressed, colors) =
      many0(argb_color)
        .parse(&decompressed_bytes)
        .map_err(|e| match e {
          nom::Err::Error(ParseError::Nom(_, kind)) => {
            nom::Err::Error(ParseError::Nom(input, kind))
          }
          nom::Err::Failure(ParseError::Nom(_, kind)) => {
            nom::Err::Failure(ParseError::Nom(input, kind))
          }
          _ => nom::Err::Failure(ParseError::DecompressZlibError),
        })?;

    Ok((input, colors))
  }
}

/// Header data of Fx.
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-16-bytes-3
#[derive(Debug)]
struct FxHeader {
  /// Total size of this model.
  data_size: u64,
  /// Fx type.
  fx_type: FxType,
}

impl FxHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(16usize).parse(input)?;

    let (_input, (data_size, fx_type)) = (le_u64, le_u8.map_res(|b| b.try_into())).parse(input)?;

    Ok((rest, Self { data_size, fx_type }))
  }
}

/// Content data of color overlay for Fx.
#[derive(Debug, Clone)]
pub struct FxColorOverlayContent {
  pub enabled: bool,
  pub color: ArgbColor,
  pub blend_mode: BlendMode,
  remaining_data: Vec<u8>,
}

impl<'a> FxColorOverlayContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;

      let (remaining_data, (enabled, color, blend_mode)) =
        (bool, argb_color, blend_mode).parse(input)?;

      Ok((
        rest,
        Self {
          enabled,
          color,
          blend_mode,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// Content data of outline for Fx.
#[derive(Debug, Clone)]
pub struct FxOutlineContent {
  pub enabled: bool,
  pub corners: Corners,
  pub color: ArgbColor,
  pub ignored_colors: Vec<ArgbColor>,
  /// Outside/inside
  /// true - outside
  /// false - inside
  pub is_outside: bool,
  pub is_water_color_on: bool,
  remaining_data: Vec<u8>,
}

impl<'a> FxOutlineContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;

      let (remainig_data, (enabled, corners, color, ignored_colors, is_outside, is_water_color_on)) =
        (
          bool,
          corners,
          argb_color,
          array_type(argb_color),
          bool,
          bool,
        )
          .parse(input)?;

      Ok((
        rest,
        Self {
          enabled,
          corners,
          color,
          ignored_colors,
          is_outside,
          is_water_color_on,
          remaining_data: remainig_data.into(),
        },
      ))
    }
  }
}

/// Content data of anti-aliasing for Fx.
#[derive(Debug, Clone)]
pub struct FxAntiAliasingContent {
  pub enabled: bool,
  /// Set of corners.
  pub corners: Vec<Corners>,
  pub intensity: f32,
  remaining_data: Vec<u8>,
}

impl<'a> FxAntiAliasingContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (remaining_data, (enabled, corners, intensity)) =
        (bool, array_type(corners), le_f32).parse(input)?;

      Ok((
        rest,
        Self {
          enabled,
          corners,
          intensity,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// Content data of pattern overlary for Fx.
#[derive(Debug, Clone)]
pub struct FxPatternOverlaryContent {
  pub enabled: bool,
  pub patterns: Vec<ArgbColor>,
  pub pattern_size: Size,
  pub opacity: f32,
  pub blend_mode: BlendMode,
  remaining_data: Vec<u8>,
}

impl<'a> FxPatternOverlaryContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (remaining_data, (enabled, patterns, pattern_size, opacity, blend_mode)) = (
        bool,
        array_type(le_u8).map_res(|data| data.chunks(4).map(|chunk| chunk.try_into()).collect()),
        size,
        le_f32,
        blend_mode,
      )
        .parse(input)?;

      Ok((
        rest,
        Self {
          enabled,
          patterns,
          pattern_size,
          opacity,
          blend_mode,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// Rendering Effect.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-7
#[derive(Debug, Clone)]
pub enum Fx {
  ColorOverlay(FxColorOverlayContent),
  Outline(FxOutlineContent),
  AntiAliasing(FxAntiAliasingContent),
  PatternOverlary(FxPatternOverlaryContent),
}

impl Fx {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = FxHeader::parse(input)?;

    match header.fx_type {
      FxType::ColorOverlay => {
        let (input, content) =
          FxColorOverlayContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Fx::ColorOverlay(content)))
      }
      FxType::Outline => {
        let (input, content) = FxOutlineContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Fx::Outline(content)))
      }
      FxType::AntiAliasing => {
        let (input, content) =
          FxAntiAliasingContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Fx::AntiAliasing(content)))
      }
      FxType::PatternOverlay => {
        let (input, content) =
          FxPatternOverlaryContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Fx::PatternOverlary(content)))
      }
    }
  }
}

/// Header data of Entry.
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-16-bytes-4
#[derive(Debug)]
struct EntryHeader {
  /// Total size of this model.
  data_size: u32,
  /// Entry type.
  entry_type: EntryType,
}

impl EntryHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(16usize).parse(input)?;

    let (_input, (data_size, entry_type)) =
      (le_u32, map_res(le_u8, |b| b.try_into())).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        entry_type,
      },
    ))
  }
}

/// Represents an entry in the artwork (layer, group, etc.), but doesn't have any actual data of the entry.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-8
#[derive(Debug, Clone)]
pub struct Entry {
  /// ID length.
  pub id_len: u8,
  /// ID.
  /// This ID is the same as the ID of `Layer`, `ReferenceLayer`, `Group`, and `TilemapLayer`.
  pub id: String,
  remaining_data: Vec<u8>,
}

impl Entry {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = EntryHeader::parse(input)?;
    let (rest, input) = take(header.data_size).parse(input)?;

    let (input, id_len) = le_u8(input)?;
    let (remaining_data, id) = dumb_string(id_len as usize).parse(input)?;

    Ok((
      rest,
      Self {
        id_len,
        id: id.to_string(),
        remaining_data: remaining_data.into(),
      },
    ))
  }
}

/// Header data of Layer.
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-32-bytes-4
#[derive(Debug)]
struct LayerHeader {
  data_size: u32,
  id_len: u8,
  name_len: u8,
  _compat: OptionSet<u8>,
}

impl LayerHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(32usize).parse(input)?;

    let (_input, (data_size, id_len, name_len, _compat)) =
      (le_u32, le_u8, le_u8, option_set_u8).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        id_len,
        name_len,
        _compat,
      },
    ))
  }
}

/// A layer with all data.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-9
#[derive(Debug, Clone)]
pub struct Layer {
  /// ID.
  /// The length is specified in the header.
  /// This is the same ID as the one in `Entry`.
  pub id: String,
  /// Name.
  /// The length is specified in the header.
  pub name: String,
  /// Frames of this layer.
  pub frames: Vec<Frame>,
  pub opacity: f16,
  pub visible: bool,
  pub locked: bool,
  pub selected: bool,
  pub alpha_locked: bool,
  /// Default: Normal
  pub blend_mode: BlendMode,
  /// Default: false
  pub linked: bool,
  /// Cropping masks, sorted in the actual order.
  /// A higher index means being above.
  /// Default: []
  pub cropping_masks: Vec<Entry>,
  /// Clipping masks, sorted in the actual order.
  /// A higher index means being above.
  /// Default: []
  pub clipping_masks: Vec<Entry>,
  /// Default: (0, 0, 0, 0)
  pub color: ArgbColor,
  /// Default: []
  pub fxs: Vec<Fx>,
  remaining_data: Vec<u8>,
}

impl Layer {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = LayerHeader::parse(input)?;
    let (rest, input) = take(header.data_size as usize).parse(input)?;

    let (
      remaining_data,
      (
        id,
        name,
        frames,
        opacity,
        visible,
        locked,
        selected,
        alpha_locked,
        blend_mode,
        linked,
        cropping_masks,
        clipping_masks,
        color,
        fxs,
      ),
    ) = (
      dumb_string(header.id_len as usize),
      dumb_string(header.name_len as usize),
      array_type(Frame::parse),
      float16,
      bool,
      bool,
      bool,
      bool,
      blend_mode,
      bool,
      array_type(Entry::parse),
      array_type(Entry::parse),
      argb_color,
      array_type(Fx::parse),
    )
      .parse(input)?;

    Ok((
      rest,
      Self {
        id: id.to_string(),
        name: name.to_string(),
        frames,
        opacity,
        visible,
        locked,
        selected,
        alpha_locked,
        blend_mode,
        linked,
        cropping_masks,
        clipping_masks,
        color,
        fxs,
        remaining_data: remaining_data.into(),
      },
    ))
  }
}

/// Header data of Group.
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-32-bytes-5
#[derive(Debug)]
struct GroupHeader {
  /// Size of this model.
  pub data_size: u32,
  /// ID length.
  pub id_len: u8,
  /// Name length.
  pub name_len: u8,
  /// Backward compatibility, always 0b00001111.
  pub _compat: OptionSet<u8>,
}

impl GroupHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(32usize).parse(input)?;

    let (_input, (data_size, id_len, name_len, _compat)) =
      (le_u32, le_u8, le_u8, option_set_u8).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        id_len,
        name_len,
        _compat,
      },
    ))
  }
}

/// A group with all data.
#[derive(Debug, Clone)]
pub struct Group {
  /// ID.
  /// This is the same ID as the one in `Entry`.
  pub id: String,
  /// Name.
  pub name: String,
  /// Child entries, sorted in the actual order.
  /// A higher index means being above.
  pub child_entries: Vec<Entry>,
  pub opacity: f16,
  pub visible: bool,
  pub content_locked: bool,
  pub selected: bool,
  pub alpha_locked: bool,
  /// Default: true.
  pub expanded: bool,
  /// Cropping masks, sorted in the actual order.
  /// A higher index means being above.
  /// Default: [].
  pub cropping_masks: Vec<Entry>,
  /// Clipping masks, sorted in the actual order.
  /// A higher index means being above.
  /// Default: [].
  pub clipping_masks: Vec<Entry>,
  /// Color.
  /// Default: (0, 0, 0, 0).
  pub color: ArgbColor,
  remaining_data: Vec<u8>,
}

impl Group {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = GroupHeader::parse(input)?;
    let (rest, input) = take(header.data_size).parse(input)?;

    let (
      remaining_data,
      (
        id,
        name,
        child_entries,
        opacity,
        visible,
        content_locked,
        selected,
        alpha_locked,
        expanded,
        cropping_masks,
        clipping_masks,
        color,
      ),
    ) = (
      dumb_string(header.id_len as usize),
      dumb_string(header.name_len as usize),
      array_type(Entry::parse),
      float16,
      bool,
      bool,
      bool,
      bool,
      bool,
      array_type(Entry::parse),
      array_type(Entry::parse),
      argb_color,
    )
      .parse(input)?;

    Ok((
      rest,
      Self {
        id: id.to_string(),
        name: name.to_string(),
        child_entries,
        opacity,
        visible,
        content_locked,
        selected,
        alpha_locked,
        expanded,
        cropping_masks,
        clipping_masks,
        color,
        remaining_data: remaining_data.into(),
      },
    ))
  }
}

/// Header data of ReferenceLayer.
/// 32 bytes
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-32-bytes-6
#[derive(Debug)]
struct ReferenceLayerHeader {
  /// Size of this model.
  data_size: u32,
  /// Length of PNG data.
  png_data_len: u64,
  /// ID length.
  id_len: u8,
  /// Name length.
  name_len: u8,
}

impl ReferenceLayerHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(32usize).parse(input)?;

    let (_input, (data_size, png_data_len, id_len, name_len)) =
      (le_u32, le_u64, le_u8, le_u8).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        png_data_len,
        id_len,
        name_len,
      },
    ))
  }
}

/// A reference layer with all data.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-11
#[derive(Debug, Clone)]
pub struct ReferenceLayer {
  /// ID.
  /// The length is specified in the header.
  /// This is the same ID as the one in `Entry`.
  pub id: String,
  /// PNG data of the reference image.
  /// Length is specified in the header.
  pub png_data: Vec<u8>,
  /// Name.
  /// The length is specified in the header.
  pub name: String,
  pub opacity: f16,
  pub visible: bool,
  pub selected: bool,
  pub bounds: Rect,
  /// Rotation angle in radians.
  /// Default: 0
  pub angle: f32,
  /// Default: (0, 0, 0, 0)
  pub color: ArgbColor,
  /// Flip axes.
  /// OptionSet<UInt8>
  /// Default: 0
  pub flip_axes: FlipAxes,
  remaining_data: Vec<u8>,
}

impl ReferenceLayer {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = ReferenceLayerHeader::parse(input)?;
    let (rest, input) = take(header.data_size).parse(input)?;

    let (
      remaining_data,
      (id, png_data, name, opacity, visible, selected, bounds, angle, color, flip_axes_bin),
    ) = (
      dumb_string(header.id_len as usize),
      take(header.png_data_len as usize),
      dumb_string(header.name_len as usize),
      float16,
      bool,
      bool,
      rect,
      le_f32,
      argb_color,
      option_set_u8,
    )
      .parse(input)?;

    Ok((
      rest,
      Self {
        id: id.to_string(),
        png_data: png_data.into(),
        name: name.to_string(),
        opacity,
        visible,
        selected,
        bounds,
        angle,
        color,
        flip_axes: FlipAxes {
          horizontal: flip_axes_bin.flag(0),
          vertical: flip_axes_bin.flag(1),
        },
        remaining_data: remaining_data.into(),
      },
    ))
  }
}

/// Header data of TilemapLayer.
/// 32 bytes
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-32-bytes-7
#[derive(Debug)]
struct TilemapLayerHeader {
  /// Size of this model.
  data_size: u32,
}

impl TilemapLayerHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(32usize).parse(input)?;

    let (_input, data_size) = le_u32.parse(input)?;

    Ok((rest, Self { data_size }))
  }
}

/// https://docs.pixquare.art/pixquare-file/binary-specs#content-6
#[derive(Debug, Clone)]
pub struct TilemapLayer {
  /// ID.
  /// This is the same ID as the one in `Entry`.
  pub id: String,
  pub tileset_id: String,
  pub name: String,
  /// Frames of this tilemap layer.
  pub frames: Vec<Frame>,
  pub opacity: f16,
  pub visible: bool,
  pub locked: bool,
  pub selected: bool,
  pub alpha_locked: bool,
  /// Default: Normal
  pub blend_mode: BlendMode,
  /// Default: false
  pub linked: bool,
  /// Default: (0, 0, 0, 0)
  pub color: ArgbColor,
  remaining_data: Vec<u8>,
}

impl TilemapLayer {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = TilemapLayerHeader::parse(input)?;
    let (rest, input) = take(header.data_size as usize).parse(input)?;

    let (
      remaining_content,
      (
        id,
        tileset_id,
        name,
        frames,
        opacity,
        visible,
        locked,
        selected,
        alpha_locked,
        blend_mode,
        linked,
        color,
      ),
    ) = (
      string,
      string,
      string,
      array_type(Frame::parse),
      float16,
      bool,
      bool,
      bool,
      bool,
      blend_mode,
      bool,
      argb_color,
    )
      .parse(input)?;

    Ok((
      rest,
      Self {
        id: id.to_string(),
        tileset_id: tileset_id.to_string(),
        name: name.to_string(),
        frames,
        opacity,
        visible,
        locked,
        selected,
        alpha_locked,
        blend_mode,
        linked,
        color,
        remaining_data: remaining_content.into(),
      },
    ))
  }
}

/// Header data of Stats.
/// 16 bytes
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-16-bytes-5
#[derive(Debug)]
struct StatsHeader {
  data_size: u16,
}

impl StatsHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(16usize).parse(input)?;

    let (_input, data_size) = le_u16.parse(input)?;

    Ok((rest, Self { data_size }))
  }
}

/// Contains data like time spent on this file, stroke count, etc.
/// Default: Empty stat, everything is 0
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-13
#[derive(Debug, Clone)]
pub struct Stats {
  /// Time spent in seconds.
  pub spent: u32,
  /// Default: 0
  pub stroke_count: u32,
  /// Default: 0
  pub undos_count: u32,
  /// Default: 0
  pub redos_count: u32,
  remaining_data: Vec<u8>,
}

impl Stats {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = StatsHeader::parse(input)?;
    let (rest, input) = take(header.data_size as usize).parse(input)?;

    let (remaining_data, (spent, stroke_count, undos_count, redos_count)) =
      (le_u32, le_u32, le_u32, le_u32).parse(input)?;

    Ok((
      rest,
      Self {
        spent,
        stroke_count,
        undos_count,
        redos_count,
        remaining_data: remaining_data.into(),
      },
    ))
  }
}

/// Settings data of grid for canvas.
/// https://docs.pixquare.art/pixquare-file/binary-specs#canvasgrid
#[derive(Debug, Clone)]
pub struct CanvasGrid {
  pub size: Size,
  pub first_color: ArgbColor,
  pub second_color: ArgbColor,
}

impl CanvasGrid {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, (size, first_color, second_color)) = (size, argb_color, argb_color).parse(input)?;

    Ok((
      input,
      Self {
        size,
        first_color,
        second_color,
      },
    ))
  }
}

/// Header data of GuideLine.
/// 14 bytes
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-14-bytes
#[derive(Debug)]
struct GuideLineHeader {
  _compat: u32,
  data_size: u16,
  guide_line_type: GuideLineType,
}

impl GuideLineHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(14usize).parse(input)?;

    let (_input, (_compat, data_size, guide_line_type)) =
      (le_u32, le_u16, map_res(le_u8, |b| b.try_into())).parse(input)?;

    Ok((
      rest,
      Self {
        _compat,
        data_size,
        guide_line_type,
      },
    ))
  }
}

/// Content data of grid type for GuideLine.
#[derive(Debug, Clone, PartialEq)]
pub struct GuideLineGridContent {
  pub size: Size,
  pub color: ArgbColor,
  pub visible: bool,
  /// Show in preview.
  pub is_shown_in_preview: bool,
  remaining_data: Vec<u8>,
}

impl<'a> GuideLineGridContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (remaining_data, (size, color, visible, is_shown_in_preview)) =
        (size, argb_color, bool, bool).parse(input)?;

      Ok((
        rest,
        Self {
          size,
          color,
          visible,
          is_shown_in_preview,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// Content data of isometric type for GuideLine.
#[derive(Debug, Clone, PartialEq)]
pub struct GuideLineIsometricContent {
  pub size: Size,
  pub color: ArgbColor,
  pub visible: bool,
  /// Show in preview.
  pub is_shown_in_preview: bool,
  remaining_data: Vec<u8>,
}

impl<'a> GuideLineIsometricContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (remaining_data, (size, color, visible, is_shown_in_preview)) =
        (size, argb_color, bool, bool).parse(input)?;

      Ok((
        rest,
        Self {
          size,
          color,
          visible,
          is_shown_in_preview,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// Content data of perspective for GuideLine.
#[derive(Debug, Clone, PartialEq)]
pub struct GuideLinePerspectiveContent {
  /// An array of x-values of perspective point coordinates.
  pub x_coordinates: Vec<f32>,
  /// An array of y-values of perspective point coordinates.
  pub y_coordinates: Vec<f32>,
  /// An array of line counts for each point.
  pub line_counts: Vec<u32>,
  /// Colors of each point.
  pub colors: Vec<ArgbColor>,
  pub visible: bool,
  /// Show in preview.
  pub is_shown_in_preview: bool,
  remaining_data: Vec<u8>,
}

impl<'a> GuideLinePerspectiveContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (
        remaining_data,
        (x_coordinates, y_coordinates, line_counts, colors, visible, is_shown_in_preview),
      ) = (
        array_type(le_f32),
        array_type(le_f32),
        array_type(le_u32),
        array_type(argb_color),
        bool,
        bool,
      )
        .parse(input)?;

      Ok((
        rest,
        Self {
          x_coordinates,
          y_coordinates,
          line_counts,
          colors,
          visible,
          is_shown_in_preview,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// Settings data of GuideLine.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-14
#[derive(Debug, Clone, PartialEq)]
pub enum GuideLine {
  Grid(GuideLineGridContent),
  Isometric(GuideLineIsometricContent),
  Perspective(GuideLinePerspectiveContent),
}

impl GuideLine {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = GuideLineHeader::parse(input)?;

    match header.guide_line_type {
      GuideLineType::Grid => {
        let (input, content) =
          GuideLineGridContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Self::Grid(content)))
      }
      GuideLineType::Isometric => {
        let (input, content) =
          GuideLineIsometricContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Self::Isometric(content)))
      }
      GuideLineType::Perspective => {
        let (input, content) =
          GuideLinePerspectiveContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Self::Perspective(content)))
      }
    }
  }
}

/// Header data of Post-processor.
/// 16 bytes
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-16-bytes-6
#[derive(Debug)]
struct PostProcessorHeader {
  /// Total size of this model.
  data_size: u16,
  processor_type: ProcessorType,
}

impl PostProcessorHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(16usize).parse(input)?;

    let (_input, (data_size, processor_type)) =
      (le_u16, map_res(le_u8, |b| b.try_into())).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        processor_type,
      },
    ))
  }
}

/// Content data of CRT for Post-processor.
#[derive(Debug, Clone)]
pub struct PostProcessorCrtContent {
  pub id: String,
  pub is_pixel_independent: bool,
  /// Scan line intensity (0-1).
  pub scan_line_intensity: f64,
  /// Glow intensity (0-1).
  pub glow_intensity: f64,
  pub enabled: bool,
  remaining_data: Vec<u8>,
}

impl<'a> PostProcessorCrtContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (
        remaining_data,
        (id, is_pixel_independent, scan_line_intensity, glow_intensity, enabled),
      ) = (string, bool, le_f64, le_f64, bool).parse(input)?;

      Ok((
        rest,
        Self {
          id: id.to_string(),
          is_pixel_independent,
          scan_line_intensity,
          glow_intensity,
          enabled,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// Content data of Vignette for Post-processor.
#[derive(Debug, Clone)]
pub struct PostProcessorVignetteContent {
  pub id: String,
  pub is_pixel_independent: bool,
  pub color: ArgbColor,
  /// Intensity (0-1).
  pub intensity: f64,
  pub enabled: bool,
  remaining_data: Vec<u8>,
}

impl<'a> PostProcessorVignetteContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (remaining_data, (id, is_pixel_independent, color, intensity, enabled)) =
        (string, bool, argb_color, le_f64, bool).parse(input)?;

      Ok((
        rest,
        Self {
          id: id.to_string(),
          is_pixel_independent,
          color,
          intensity,
          enabled,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// Content data of Bloom for Post-processor.
#[derive(Debug, Clone)]
pub struct PostProcessorBloomContent {
  pub id: String,
  pub is_pixel_independent: bool,
  /// Threshold (0-1).
  pub threshold: f64,
  /// Intensity (0-1).
  pub intensity: f64,
  /// Radius (0-1).
  pub radius: f64,
  pub enabled: bool,
  remaining_data: Vec<u8>,
}

impl<'a> PostProcessorBloomContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (remaining_data, (id, is_pixel_independent, threshold, intensity, radius, enabled)) =
        (string, bool, le_f64, le_f64, le_f64, bool).parse(input)?;

      Ok((
        rest,
        Self {
          id: id.to_string(),
          is_pixel_independent,
          threshold,
          intensity,
          radius,
          enabled,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// Content data of Round pixel for Post-processor.
#[derive(Debug, Clone)]
pub struct PostProcessorRoundPixelContent {
  pub id: String,
  pub is_contiguous: bool,
  pub background_color: ArgbColor,
  /// Intensity (0-1).
  pub intensity: f64,
  pub enabled: bool,
  remaining_data: Vec<u8>,
}

impl<'a> PostProcessorRoundPixelContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (remaining_data, (id, is_contiguous, background_color, intensity, enabled)) =
        (string, bool, argb_color, le_f64, bool).parse(input)?;

      Ok((
        rest,
        Self {
          id: id.to_string(),
          is_contiguous,
          background_color,
          intensity,
          enabled,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// Settings data of Post-processor.
#[derive(Debug, Clone)]
pub enum PostProcessor {
  Crt(PostProcessorCrtContent),
  Vignette(PostProcessorVignetteContent),
  Bloom(PostProcessorBloomContent),
  RoundPixel(PostProcessorRoundPixelContent),
}

impl PostProcessor {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = PostProcessorHeader::parse(input)?;

    match header.processor_type {
      ProcessorType::Crt => {
        let (input, content) =
          PostProcessorCrtContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Self::Crt(content)))
      }
      ProcessorType::Vignette => {
        let (input, content) =
          PostProcessorVignetteContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Self::Vignette(content)))
      }
      ProcessorType::Bloom => {
        let (input, content) =
          PostProcessorBloomContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Self::Bloom(content)))
      }
      ProcessorType::RoundPixel => {
        let (input, content) =
          PostProcessorRoundPixelContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Self::RoundPixel(content)))
      }
    }
  }
}

/// Header data of Modifier.
/// 16 bytes
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-16-bytes-7
#[derive(Debug)]
struct ModifierHeader {
  data_size: u64,
  modifier_type: ModifierType,
}

impl ModifierHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(16usize).parse(input)?;

    let (_input, (data_size, modifier_type)) =
      (le_u64, map_res(le_u8, |b| b.try_into())).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        modifier_type,
      },
    ))
  }
}

/// Content data of Animation speed multiplier for Modifier.
#[derive(Debug, Clone)]
pub struct ModifierAnimationSpeedMultiplierContent {
  id: String,
  enabled: bool,
  /// The multiplier.
  /// 2 means twice as fast, 0.5 means 2 times slower.
  multiplier: f32,
  remaining_data: Vec<u8>,
}

impl<'a> ModifierAnimationSpeedMultiplierContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (remaining_data, (id, enabled, multiplier)) = (string, bool, le_f32).parse(input)?;

      Ok((
        rest,
        Self {
          id: id.to_string(),
          enabled,
          multiplier,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// https://docs.pixquare.art/pixquare-file/binary-specs#content-16
#[derive(Debug, Clone)]
pub enum Modifier {
  AnimationSpeedMultiplier(ModifierAnimationSpeedMultiplierContent),
}

impl Modifier {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = ModifierHeader::parse(input)?;

    match header.modifier_type {
      ModifierType::AnimationSpeedMultiplier => {
        let (input, content) =
          ModifierAnimationSpeedMultiplierContent::parser(header.data_size as usize)
            .parse(input)?;

        Ok((input, Self::AnimationSpeedMultiplier(content)))
      }
    }
  }
}

/// Header data of PaletteOrganization.
/// 16 bytes
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-16-bytes-8
#[derive(Debug)]
struct PaletteOrganizationHeader {
  /// Total size of this model.
  data_size: u16,
  organization_type: PaletteOrganizationType,
}

impl PaletteOrganizationHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(16usize).parse(input)?;

    let (_input, (data_size, organization_type)) =
      (le_u16, map_res(le_u8, |b| b.try_into())).parse(input)?;

    Ok((
      rest,
      Self {
        data_size,
        organization_type,
      },
    ))
  }
}

/// Content data of Anywhere for PaletteOrganization.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteOrganizationAnywhereContent {
  pub size: Size,
  /// The arrangement that fits into the size above.
  /// true - empty space
  /// false - color space
  pub arrangements: Vec<bool>,
  remaining_data: Vec<u8>,
}

impl<'a> PaletteOrganizationAnywhereContent {
  fn parser(
    data_size: usize,
  ) -> impl Parser<&'a [u8], Output = Self, Error = ParseError<&'a [u8]>> + Clone {
    move |input| {
      let (rest, input) = take(data_size).parse(input)?;
      let (input, size) = size.parse(input)?;
      let (remaining_data, arrangements) =
        type_n(bool, (size.width as usize) * (size.height as usize)).parse(input)?;

      Ok((
        rest,
        Self {
          size,
          arrangements,
          remaining_data: remaining_data.into(),
        },
      ))
    }
  }
}

/// https://docs.pixquare.art/pixquare-file/binary-specs#content-17
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteOrganization {
  Packed,
  Anywhere(PaletteOrganizationAnywhereContent),
}

impl PaletteOrganization {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = PaletteOrganizationHeader::parse(input)?;

    match header.organization_type {
      PaletteOrganizationType::Packed => Ok((input, Self::Packed)),
      PaletteOrganizationType::Anywhere => {
        let (input, content) =
          PaletteOrganizationAnywhereContent::parser(header.data_size as usize).parse(input)?;

        Ok((input, Self::Anywhere(content)))
      }
    }
  }
}

/// Header data of Artwork.
/// https://docs.pixquare.art/pixquare-file/binary-specs#header-64-bytes
#[derive(Debug)]
struct ArtworkHeader {
  /// Total size of the file.
  pub file_size: u64,
  /// Length of `id`.
  pub id_len: u8,
  /// Backward compatibility. Always 2.
  _compat_data: u16,
}

impl ArtworkHeader {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (rest, input) = take(64usize).parse(input)?;
    let (input, file_size) = le_u64(input)?;
    let (input, id_len) = le_u8(input)?;
    let (_input, compat) = le_u16(input)?;

    Ok((
      rest,
      Self {
        file_size,
        id_len,
        _compat_data: compat,
      },
    ))
  }
}

/// This is the data of .px files.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content-18
#[derive(Debug, Clone)]
pub struct Artwork {
  /// ID.
  pub id: String,
  /// Canvas size.
  pub canvas_size: Size,
  /// Entries at the root level of the artwork.
  /// Sorted in the actual order.
  /// A higher index means being above.
  pub entries: Vec<Entry>,
  pub groups: Vec<Group>,
  pub layers: Vec<Layer>,
  pub frame_contents: Vec<FrameContent>,
  pub palette: Vec<ArgbColor>,
  /// Default: []
  pub reference_layers: Vec<ReferenceLayer>,
  /// Default: []
  pub reference_images: Vec<Vec<u8>>,
  /// Default: []
  pub symmetry_lines: Vec<SymmetryLine>,
  /// Default: []
  pub tags: Vec<Tag>,
  /// Default: []
  pub tilesets: Vec<Tileset>,
  /// Default: []
  pub tilemap_layers: Vec<TilemapLayer>,
  /// Default: []
  pub tilemap_frame_contents: Vec<TilemapFrameContent>,
  /// Default: RGB
  pub color_depth: ColorDepth,
  /// Some stats of this file.
  pub stats: Stats,
  /// Ignore this byte.
  _unused: u8,
  /// Canvas grid config.
  /// Default: 16 x 16 Grid with blue color.
  pub canvas_grid: CanvasGrid,
  /// Guide line config.
  /// Default: 16 x 16 Grid with blue color.
  pub guide_line: GuideLine,
  /// Default: []
  pub post_processors: Vec<PostProcessor>,
  /// Default: Packed
  pub palette_organization: PaletteOrganization,
  /// Should record timelapse.
  pub is_need_timelapse: bool,
  /// Default: 0
  pub tiled_corners: Corners,
  /// Default: []
  pub modifiers: Vec<Modifier>,
  remaining_data: Vec<u8>,
}

impl<'a> Artwork {
  pub fn read(buf: &'a [u8]) -> Result<Self, ParseError<&'a [u8]>> {
    let (_rest, artwork) = Self::parse(buf).map_err(|nom_err| match nom_err {
      nom::Err::Error(e) | nom::Err::Failure(e) => e,
      nom::Err::Incomplete(_) => ParseError::Nom(buf, ErrorKind::Eof),
    })?;

    Ok(artwork)
  }

  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = ArtworkHeader::parse(input)?;

    let (
      remaining_data,
      (
        id,
        canvas_size,
        (
          entries,
          groups,
          layers,
          frame_contents,
          palette,
          reference_layers,
          reference_images,
          symmetry_lines,
          tags,
          tilesets,
          tilemap_layers,
          tilemap_frame_contents,
        ),
        color_depth,
        stats,
        _unused,
        canvas_grid,
        guide_line,
        post_processors,
        palette_organization,
        is_need_timelapse,
        tiled_corners,
        modifiers,
      ),
    ) = (
      dumb_string(header.id_len as usize),
      size,
      (
        array_type(Entry::parse),
        array_type(Group::parse),
        array_type(Layer::parse),
        array_type(FrameContent::parse),
        array_type(argb_color),
        array_type(ReferenceLayer::parse),
        array_type(array_type(le_u8)),
        array_type(SymmetryLine::parse),
        array_type(Tag::parse),
        array_type(Tileset::parse),
        array_type(TilemapLayer::parse),
        array_type(TilemapFrameContent::parse),
      ),
      map_res(le_u8, |b| b.try_into()),
      Stats::parse,
      le_u8,
      CanvasGrid::parse,
      GuideLine::parse,
      array_type(PostProcessor::parse),
      PaletteOrganization::parse,
      bool,
      corners,
      array_type(Modifier::parse),
    )
      .parse(input)?;

    Ok((
      remaining_data,
      Self {
        id: id.to_string(),
        canvas_size,
        entries,
        groups,
        layers,
        frame_contents,
        palette,
        reference_layers,
        reference_images,
        symmetry_lines,
        tags,
        tilesets,
        tilemap_layers,
        tilemap_frame_contents,
        color_depth,
        stats,
        _unused,
        canvas_grid,
        guide_line,
        post_processors,
        palette_organization,
        is_need_timelapse,
        tiled_corners,
        modifiers,
        remaining_data: remaining_data.into(),
      },
    ))
  }
}

#[cfg(test)]
mod tests {
  use crate::composite_type::Coordinate;

  use super::*;

  #[test]
  fn test_read_file_data() {
    let path = "assets/fixtures/simple.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    assert_eq!(
      artwork.canvas_size,
      Size {
        width: 63,
        height: 63
      }
    );
    assert!(artwork.groups.is_empty());
    assert_eq!(artwork.layers[0].name, "base");
    assert_eq!(artwork.layers[1].name, "Layer 1");
    assert!(artwork.reference_layers.is_empty());
    assert!(artwork.reference_images.is_empty());
    assert!(artwork.symmetry_lines.is_empty());
    assert!(artwork.tags.is_empty());
    assert!(artwork.tilesets.is_empty());
    assert!(artwork.tilemap_layers.is_empty());
    assert!(artwork.tilemap_frame_contents.is_empty());
    assert_eq!(artwork.color_depth, ColorDepth::Rgb);
    assert_eq!(artwork.stats.spent, 504);
    assert_eq!(artwork.stats.stroke_count, 83);
    assert_eq!(artwork.stats.undos_count, 23);
    assert_eq!(artwork.stats.redos_count, 2);
    assert_eq!(artwork._unused, 0);
    assert_eq!(
      artwork.canvas_grid.size,
      Size {
        width: 16,
        height: 16,
      }
    );
    assert_eq!(
      artwork.canvas_grid.first_color,
      ArgbColor {
        r: 204,
        g: 204,
        b: 204,
        a: 255,
      }
    );
    assert_eq!(
      artwork.canvas_grid.second_color,
      ArgbColor {
        r: 230,
        g: 230,
        b: 230,
        a: 255,
      }
    );
    assert_eq!(
      artwork.guide_line,
      GuideLine::Grid(GuideLineGridContent {
        size: Size {
          width: 16,
          height: 16
        },
        color: ArgbColor {
          r: 0,
          g: 0,
          b: 255,
          a: 255
        },
        visible: false,
        is_shown_in_preview: false,
        remaining_data: vec![],
      })
    );
    assert!(artwork.post_processors.is_empty());
    assert_eq!(artwork.palette_organization, PaletteOrganization::Packed);
    assert_eq!(artwork.is_need_timelapse, false);
    assert_eq!(
      artwork.tiled_corners,
      Corners {
        top_left: false,
        top: false,
        top_right: false,
        right: false,
        bottom_right: false,
        bottom: false,
        bottom_left: false,
        left: false,
      }
    );
    assert!(artwork.modifiers.is_empty());
  }

  #[test]
  fn test_parse_entry_data() {
    let path = "assets/fixtures/entry.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    assert_eq!(artwork.entries.len(), 5);
    assert!(
      artwork
        .layers
        .iter()
        .any(|l| l.id == artwork.entries[0].id && l.name == "Layer 1")
    );
    assert!(
      artwork
        .layers
        .iter()
        .any(|l| l.id == artwork.entries[1].id && l.name == "Layer 2")
    );
    assert!(
      artwork
        .groups
        .iter()
        .any(|g| g.id == artwork.entries[2].id && g.name == "Group 1")
    );
    assert_eq!(artwork.entries[3].id, artwork.reference_layers[0].id);
    assert_eq!(artwork.entries[4].id, artwork.tilemap_layers[0].id);
  }

  #[test]
  fn test_parse_group_data() {
    let path = "assets/fixtures/group.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    assert_eq!(artwork.groups.len(), 3);
    // Group 1
    assert_eq!(artwork.groups[0].name, "Group 1");
    assert_eq!(artwork.groups[0].child_entries.len(), 3);
    assert_eq!(artwork.groups[0].opacity, f16::from_f32(1.0));
    assert_eq!(artwork.groups[0].visible, true);
    assert_eq!(artwork.groups[0].content_locked, false);
    assert_eq!(artwork.groups[0].selected, true);
    assert_eq!(artwork.groups[0].alpha_locked, true);
    assert_eq!(artwork.groups[0].expanded, false);
    assert!(artwork.groups[0].cropping_masks.is_empty());
    assert_eq!(artwork.groups[0].clipping_masks.len(), 1);
    assert_eq!(artwork.groups[0].color, ArgbColor::default());
    // Group2
    assert_eq!(artwork.groups[1].name, "Group 2");
    assert_eq!(artwork.groups[1].child_entries.len(), 1);
    assert_eq!(artwork.groups[1].opacity, f16::from_f32(1.0));
    assert_eq!(artwork.groups[1].visible, false);
    assert_eq!(artwork.groups[1].content_locked, true);
    assert_eq!(artwork.groups[1].selected, false);
    assert_eq!(artwork.groups[1].alpha_locked, false);
    assert_eq!(artwork.groups[1].expanded, true);
    assert!(artwork.groups[1].cropping_masks.is_empty());
    assert!(artwork.groups[1].clipping_masks.is_empty());
    assert_eq!(artwork.groups[1].color, ArgbColor::default());
    // Group 3
    assert_eq!(artwork.groups[2].name, "Group 3");
    assert_eq!(artwork.groups[2].child_entries.len(), 1);
    assert_eq!(artwork.groups[2].opacity, f16::from_f32(0.77));
    assert_eq!(artwork.groups[2].visible, true);
    assert_eq!(artwork.groups[2].content_locked, false);
    assert_eq!(artwork.groups[2].selected, false);
    assert_eq!(artwork.groups[2].alpha_locked, false);
    assert_eq!(artwork.groups[2].expanded, true);
    assert!(artwork.groups[2].cropping_masks.is_empty());
    assert!(artwork.groups[2].clipping_masks.is_empty());
    assert_eq!(artwork.groups[2].color, ArgbColor::new(143, 42, 42, 225));
  }

  #[test]
  fn test_parse_layer_data() {
    let path = "assets/fixtures/layer.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    assert_eq!(artwork.layers.len(), 7);
    // Layer 1
    assert_eq!(artwork.layers[0].name, "Layer 1");
    assert_eq!(artwork.layers[0].frames.len(), 1);
    assert_eq!(artwork.layers[0].opacity, f16::from_f32(1.0));
    assert_eq!(artwork.layers[0].visible, true);
    assert_eq!(artwork.layers[0].locked, false);
    assert_eq!(artwork.layers[0].selected, false);
    assert_eq!(artwork.layers[0].alpha_locked, false);
    assert_eq!(artwork.layers[0].blend_mode, BlendMode::Normal);
    assert_eq!(artwork.layers[0].linked, false);
    assert!(artwork.layers[0].cropping_masks.is_empty());
    assert!(artwork.layers[0].clipping_masks.is_empty());
    assert_eq!(artwork.layers[0].color, ArgbColor::new(98, 122, 93, 255));
    assert!(artwork.layers[0].fxs.is_empty());
    // Layer 2
    assert_eq!(artwork.layers[1].name, "Layer 2");
    assert_eq!(artwork.layers[1].frames.len(), 1);
    assert_eq!(artwork.layers[1].opacity, f16::from_f32(1.0));
    assert_eq!(artwork.layers[1].visible, true);
    assert_eq!(artwork.layers[1].locked, true);
    assert_eq!(artwork.layers[1].selected, false);
    assert_eq!(artwork.layers[1].alpha_locked, false);
    assert_eq!(artwork.layers[1].blend_mode, BlendMode::Normal);
    assert_eq!(artwork.layers[1].linked, true);
    assert!(artwork.layers[1].cropping_masks.is_empty());
    assert!(artwork.layers[1].clipping_masks.is_empty());
    assert_eq!(artwork.layers[1].color, ArgbColor::default());
    assert!(artwork.layers[1].fxs.is_empty());
    // Layer 3
    assert_eq!(artwork.layers[2].name, "Layer 3");
    assert_eq!(artwork.layers[2].frames.len(), 1);
    assert_eq!(artwork.layers[2].opacity, f16::from_f32(1.0));
    assert_eq!(artwork.layers[2].visible, false);
    assert_eq!(artwork.layers[2].locked, false);
    assert_eq!(artwork.layers[2].selected, false);
    assert_eq!(artwork.layers[2].alpha_locked, false);
    assert_eq!(artwork.layers[2].blend_mode, BlendMode::Normal);
    assert_eq!(artwork.layers[2].linked, false);
    assert!(artwork.layers[2].cropping_masks.is_empty());
    assert!(artwork.layers[2].clipping_masks.is_empty());
    assert_eq!(artwork.layers[2].color, ArgbColor::default());
    assert!(artwork.layers[2].fxs.is_empty());
    // Layer 4
    assert_eq!(artwork.layers[3].name, "Layer 4");
    assert_eq!(artwork.layers[3].frames.len(), 1);
    assert_eq!(artwork.layers[3].opacity, f16::from_f32(0.58));
    assert_eq!(artwork.layers[3].visible, true);
    assert_eq!(artwork.layers[3].locked, false);
    assert_eq!(artwork.layers[3].selected, true);
    assert_eq!(artwork.layers[3].alpha_locked, true);
    assert_eq!(artwork.layers[3].blend_mode, BlendMode::Normal);
    assert_eq!(artwork.layers[3].linked, false);
    assert!(artwork.layers[3].cropping_masks.is_empty());
    assert_eq!(artwork.layers[3].clipping_masks.len(), 1);
    assert_eq!(artwork.layers[3].color, ArgbColor::default());
    assert!(artwork.layers[3].fxs.is_empty());
    // Layer 5
    assert_eq!(artwork.layers[4].name, "Layer 5");
    assert_eq!(artwork.layers[4].frames.len(), 1);
    assert_eq!(artwork.layers[4].opacity, f16::from_f32(1.0));
    assert_eq!(artwork.layers[4].visible, true);
    assert_eq!(artwork.layers[4].locked, false);
    assert_eq!(artwork.layers[4].selected, false);
    assert_eq!(artwork.layers[4].alpha_locked, false);
    assert_eq!(artwork.layers[4].blend_mode, BlendMode::Normal);
    assert_eq!(artwork.layers[4].linked, false);
    assert!(artwork.layers[4].cropping_masks.is_empty());
    assert!(artwork.layers[4].clipping_masks.is_empty());
    assert_eq!(artwork.layers[4].color, ArgbColor::default());
    assert!(artwork.layers[4].fxs.is_empty());
    // Layer 6
    assert_eq!(artwork.layers[5].name, "Layer 6");
    assert_eq!(artwork.layers[5].frames.len(), 1);
    assert_eq!(artwork.layers[5].opacity, f16::from_f32(1.0));
    assert_eq!(artwork.layers[5].visible, true);
    assert_eq!(artwork.layers[5].locked, false);
    assert_eq!(artwork.layers[5].selected, false);
    assert_eq!(artwork.layers[5].alpha_locked, false);
    assert_eq!(artwork.layers[5].blend_mode, BlendMode::Overlay);
    assert_eq!(artwork.layers[5].linked, false);
    assert_eq!(artwork.layers[5].cropping_masks.len(), 1);
    assert!(artwork.layers[5].clipping_masks.is_empty());
    assert_eq!(artwork.layers[5].color, ArgbColor::default());
    assert!(artwork.layers[5].fxs.is_empty());
    // Layer 7
    assert_eq!(artwork.layers[6].name, "Layer 7");
    assert_eq!(artwork.layers[6].frames.len(), 1);
    assert_eq!(artwork.layers[6].opacity, f16::from_f32(1.0));
    assert_eq!(artwork.layers[6].visible, true);
    assert_eq!(artwork.layers[6].locked, false);
    assert_eq!(artwork.layers[6].selected, false);
    assert_eq!(artwork.layers[6].alpha_locked, false);
    assert_eq!(artwork.layers[6].blend_mode, BlendMode::Normal);
    assert_eq!(artwork.layers[6].linked, false);
    assert!(artwork.layers[6].cropping_masks.is_empty());
    assert!(artwork.layers[6].clipping_masks.is_empty());
    assert_eq!(artwork.layers[6].color, ArgbColor::default());
    assert!(artwork.layers[6].fxs.is_empty());
  }

  #[test]
  fn test_parse_layer_fx_data() {
    let path = "assets/fixtures/layer-fx.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    assert_eq!(artwork.layers.len(), 3);
    assert_eq!(artwork.layers[0].name, "Layer 1");
    assert_eq!(artwork.layers[0].fxs.len(), 1);
    let Fx::PatternOverlary(fx) = &artwork.layers[0].fxs[0] else {
      panic!("expected Fx::PatternOverlary");
    };
    assert_eq!(fx.enabled, true);
    assert_eq!(
      fx.patterns.len(),
      (fx.pattern_size.width * fx.pattern_size.height) as usize
    );
    assert_eq!(fx.pattern_size, Size::new(8, 8));
    assert_eq!(fx.opacity, 0.8);
    assert_eq!(fx.blend_mode, BlendMode::Overlay);

    assert_eq!(artwork.layers[1].name, "Layer 2");
    assert_eq!(artwork.layers[1].fxs.len(), 1);
    let Fx::ColorOverlay(fx) = &artwork.layers[1].fxs[0] else {
      panic!(
        "expected Fx::ColorOverlay, actual {:?}",
        &artwork.layers[1].fxs[0]
      );
    };
    assert_eq!(fx.enabled, true);
    assert_eq!(fx.color, ArgbColor::new(183, 166, 167, 212));
    assert_eq!(fx.blend_mode, BlendMode::Screen);

    assert_eq!(artwork.layers[2].name, "Layer 3");
    assert_eq!(artwork.layers[2].fxs.len(), 2);
    let Fx::Outline(fx) = &artwork.layers[2].fxs[0] else {
      panic!(
        "expected Fx::Outline, actual {:?}",
        &artwork.layers[2].fxs[0]
      );
    };
    assert_eq!(fx.enabled, true);
    assert_eq!(
      fx.corners,
      Corners {
        top_left: true,
        top: true,
        right: true,
        left: true,
        bottom: true,
        ..Default::default()
      }
    );
    assert_eq!(fx.ignored_colors.len(), 1);
    assert_eq!(fx.ignored_colors[0], ArgbColor::new(121, 58, 128, 255));
    assert_eq!(fx.is_outside, true);
    assert_eq!(fx.is_water_color_on, true);

    let Fx::AntiAliasing(fx) = &artwork.layers[2].fxs[1] else {
      panic!(
        "expected Fx::AntiAliasing, actual {:?}",
        &artwork.layers[2].fxs[1]
      );
    };
    assert_eq!(fx.enabled, true);
    assert_eq!(fx.corners.len(), 7);
    assert_eq!(
      fx.corners[0],
      Corners {
        top: true,
        left: true,
        bottom_left: true,
        ..Default::default()
      }
    );
    assert_eq!(
      fx.corners[1],
      Corners {
        top: true,
        right: true,
        bottom_right: true,
        ..Default::default()
      }
    );
    assert_eq!(
      fx.corners[2],
      Corners {
        top_left: true,
        left: true,
        bottom: true,
        ..Default::default()
      }
    );
    assert_eq!(
      fx.corners[3],
      Corners {
        top_right: true,
        right: true,
        bottom: true,
        ..Default::default()
      }
    );
    assert_eq!(
      fx.corners[4],
      Corners {
        top: true,
        top_right: true,
        left: true,
        ..Default::default()
      }
    );
    assert_eq!(
      fx.corners[5],
      Corners {
        left: true,
        bottom: true,
        bottom_right: true,
        ..Default::default()
      }
    );
    assert_eq!(
      fx.corners[6],
      Corners {
        right: true,
        bottom_left: true,
        bottom: true,
        ..Default::default()
      }
    );
    assert_eq!(fx.intensity, 0.5);
  }

  #[test]
  fn test_parse_layer_frame_data() {
    let path = "assets/fixtures/layer-frame.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    assert_eq!(artwork.layers.len(), 2);
    assert_eq!(artwork.layers[0].frames.len(), 3);
    assert_eq!(artwork.layers[1].frames.len(), 3);
    let frame1 = &artwork.layers[0].frames[0];
    assert_eq!(frame1.duration, 155);
    assert_eq!(frame1.selected, false);
    assert_eq!(frame1.opacity, f16::from_f32(2.0));
    assert_eq!(frame1.z_index, 0);
    let frame2 = &artwork.layers[0].frames[1];
    assert_eq!(frame2.duration, 155);
    assert_eq!(frame2.selected, true);
    assert_eq!(frame2.opacity, f16::from_f32(2.0));
    assert_eq!(frame2.z_index, 0);
    let frame3 = &artwork.layers[0].frames[2];
    assert_eq!(frame3.duration, 100);
    assert_eq!(frame3.selected, false);
    assert_eq!(frame3.opacity, f16::from_f32(2.0));
    assert_eq!(frame3.z_index, 0);
    assert_eq!(frame1.id, artwork.layers[1].frames[0].id);
    assert_eq!(frame2.id, artwork.layers[1].frames[1].id);
    assert_eq!(frame3.id, artwork.layers[1].frames[2].id);
  }

  #[test]
  fn test_parse_frame_content_data() {
    let path = "assets/fixtures/frame_content.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    assert_eq!(artwork.layers[0].frames.len(), 4);
    assert_eq!(artwork.layers[1].frames.len(), 4);
    assert_eq!(artwork.layers[2].frames.len(), 4);
    assert_ne!(
      artwork.layers[0].frames[0].content_id,
      artwork.layers[0].frames[1].content_id
    );
    assert_eq!(
      artwork.layers[0].frames[0].content_id,
      artwork.layers[0].frames[2].content_id
    );
    assert_ne!(
      artwork.layers[0].frames[0].content_id,
      artwork.layers[0].frames[3].content_id
    );
    assert_ne!(
      artwork.layers[0].frames[1].content_id,
      artwork.layers[0].frames[2].content_id
    );
    assert_ne!(
      artwork.layers[0].frames[1].content_id,
      artwork.layers[0].frames[3].content_id
    );
    assert_ne!(
      artwork.layers[0].frames[2].content_id,
      artwork.layers[0].frames[3].content_id
    );
    assert_ne!(
      artwork.layers[1].frames[0].content_id,
      artwork.layers[1].frames[1].content_id
    );
    assert_ne!(
      artwork.layers[1].frames[0].content_id,
      artwork.layers[1].frames[2].content_id
    );
    assert_ne!(
      artwork.layers[1].frames[0].content_id,
      artwork.layers[1].frames[3].content_id
    );
    assert_eq!(
      artwork.layers[1].frames[1].content_id,
      artwork.layers[1].frames[2].content_id
    );
    assert_ne!(
      artwork.layers[1].frames[1].content_id,
      artwork.layers[1].frames[3].content_id
    );
    assert_ne!(
      artwork.layers[1].frames[2].content_id,
      artwork.layers[1].frames[3].content_id
    );
    assert_ne!(
      artwork.layers[2].frames[0].content_id,
      artwork.layers[2].frames[1].content_id
    );
    assert_ne!(
      artwork.layers[2].frames[0].content_id,
      artwork.layers[2].frames[2].content_id
    );
    assert_ne!(
      artwork.layers[2].frames[0].content_id,
      artwork.layers[2].frames[3].content_id
    );
    assert_ne!(
      artwork.layers[2].frames[1].content_id,
      artwork.layers[2].frames[2].content_id
    );
    assert_ne!(
      artwork.layers[2].frames[1].content_id,
      artwork.layers[2].frames[3].content_id
    );
    assert_ne!(
      artwork.layers[2].frames[2].content_id,
      artwork.layers[2].frames[3].content_id
    );
    // collect unique content_ids
    let mut uniq_ids: Vec<&str> = vec![];
    for i in 0..artwork.layers.len() {
      for j in 0..artwork.layers[i].frames.len() {
        if uniq_ids
          .iter()
          .all(|id| id != &artwork.layers[i].frames[j].content_id)
        {
          uniq_ids.push(&artwork.layers[i].frames[j].content_id);
        }
      }
    }
    assert_eq!(uniq_ids.len(), artwork.frame_contents.len());
    assert!(
      get_frame_content_by_indices(&artwork, 0, 0)
        .colors
        .iter()
        .any(|color| color != &ArgbColor::default())
    );
    assert!(
      get_frame_content_by_indices(&artwork, 0, 1)
        .colors
        .iter()
        .all(|color| color == &ArgbColor::default())
    );
    assert!(
      get_frame_content_by_indices(&artwork, 0, 3)
        .colors
        .iter()
        .all(|color| color == &ArgbColor::default())
    );
    assert!(
      get_frame_content_by_indices(&artwork, 1, 0)
        .colors
        .iter()
        .all(|color| color == &ArgbColor::default())
    );
    assert!(
      get_frame_content_by_indices(&artwork, 1, 0)
        .colors
        .iter()
        .all(|color| color == &ArgbColor::default())
    );
    assert!(
      get_frame_content_by_indices(&artwork, 1, 1)
        .colors
        .iter()
        .any(|color| color != &ArgbColor::default())
    );
    assert!(
      get_frame_content_by_indices(&artwork, 1, 3)
        .colors
        .iter()
        .all(|color| color == &ArgbColor::default())
    );
    assert!(
      get_frame_content_by_indices(&artwork, 2, 0)
        .colors
        .iter()
        .all(|color| color == &ArgbColor::default())
    );
    assert!(
      get_frame_content_by_indices(&artwork, 2, 1)
        .colors
        .iter()
        .any(|color| color != &ArgbColor::default())
    );
    assert!(
      get_frame_content_by_indices(&artwork, 2, 2)
        .colors
        .iter()
        .all(|color| color == &ArgbColor::default())
    );
    assert!(
      get_frame_content_by_indices(&artwork, 2, 3)
        .colors
        .iter()
        .all(|color| color == &ArgbColor::default())
    );
  }

  #[test]
  fn test_parse_palette_data() {
    let path = "assets/fixtures/palette.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    assert_eq!(artwork.palette.len(), 4);
    assert_eq!(artwork.palette[0], ArgbColor::new(0, 0, 0, 255));
    assert_eq!(artwork.palette[1], ArgbColor::new(255, 255, 255, 255));
    assert_eq!(
      artwork.palette[2],
      ArgbColor::post_multiply(20, 160, 46, 179)
    );
    assert_eq!(artwork.palette[3], ArgbColor::new(255, 0, 0, 255));
    assert_eq!(artwork.palette_organization, PaletteOrganization::Packed);
  }

  #[test]
  fn test_parse_palette_anywhere_data() {
    let path = "assets/fixtures/palette-anywhere.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    assert_eq!(artwork.palette.len(), 7);
    let PaletteOrganization::Anywhere(content) = &artwork.palette_organization else {
      panic!("expected PaletteOrganization::Anywhere");
    };
    assert_eq!(content.size, Size::new(9, 2));
    assert_eq!(
      content.arrangements,
      vec![
        false, true, true, false, true, false, false, false, true, false, true, true, true, false,
        false, false, false, false,
      ]
    );
  }

  #[test]
  fn test_parse_reference_layer_data() {
    let path = "assets/fixtures/reference_layer.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    assert_eq!(artwork.reference_layers.len(), 2);
    // Reference Layer 1
    assert_eq!(artwork.reference_layers[0].name, "Reference Layer 1");
    assert!(artwork.reference_layers[0].png_data.len() > 0);
    assert_eq!(
      artwork.reference_layers[0].opacity,
      f16::from_f32(0.7001953)
    );
    assert_eq!(artwork.reference_layers[0].visible, false);
    assert_eq!(artwork.reference_layers[0].selected, false);
    assert_eq!(
      artwork.reference_layers[0].bounds,
      Rect {
        origin: Coordinate { x: 16, y: 0 },
        size: Size {
          width: 48,
          height: 48,
        },
      }
    );
    assert_eq!(artwork.reference_layers[0].angle, 0.0);
    assert_eq!(
      artwork.reference_layers[0].color,
      ArgbColor::new(255, 0, 0, 255)
    );
    assert_eq!(artwork.reference_layers[0].flip_axes.vertical, false);
    assert_eq!(artwork.reference_layers[0].flip_axes.horizontal, false);
    // Reference Layer 2
    assert_eq!(artwork.reference_layers[1].name, "Reference Layer 2");
    assert!(artwork.reference_layers[1].png_data.len() > 0);
    assert_eq!(artwork.reference_layers[1].opacity, f16::from_f32(1.0));
    assert_eq!(artwork.reference_layers[1].visible, true);
    assert_eq!(artwork.reference_layers[1].selected, false);
    assert_eq!(
      artwork.reference_layers[1].bounds,
      Rect {
        origin: Coordinate { x: 0, y: 19 },
        size: Size {
          width: 45,
          height: 45,
        },
      }
    );
    assert_eq!(artwork.reference_layers[1].angle, 0.8069841);
    assert_eq!(artwork.reference_layers[1].color, ArgbColor::default());
    assert_eq!(artwork.reference_layers[1].flip_axes.vertical, false);
    assert_eq!(artwork.reference_layers[1].flip_axes.horizontal, true);
  }

  #[test]
  fn test_parse_reference_image_data() {
    let path = "assets/fixtures/reference_image.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    assert_eq!(artwork.reference_images.len(), 2);
  }

  #[test]
  fn test_parse_symmetry_line_data() {
    let path = "assets/fixtures/symmetry_line.px";
    let file_data = std::fs::read(path).unwrap();
    let file = Artwork::read(&file_data);

    assert!(file.is_ok());

    let artwork = file.unwrap();
    println!("{:?}", artwork.symmetry_lines);
    assert_eq!(artwork.symmetry_lines.len(), 2);
    assert_eq!(
      artwork.symmetry_lines[0].color,
      ArgbColor::new(0, 255, 58, 255)
    );
    assert_eq!(artwork.symmetry_lines[0].selected, false);
    assert_eq!(artwork.symmetry_lines[0].enabled, true);
    assert_eq!(artwork.symmetry_lines[0].origin_x, 16.);
    assert_eq!(artwork.symmetry_lines[0].origin_y, 32.5);
    assert_eq!(artwork.symmetry_lines[0].angle, 2.3561945);
    assert_eq!(artwork.symmetry_lines[0].segment_count, 2);
    assert_eq!(
      artwork.symmetry_lines[0].symmetry_type,
      SymmetryType::Mirror
    );
    assert_eq!(
      artwork.symmetry_lines[1].color,
      ArgbColor::new(0, 8, 255, 255)
    );
    assert_eq!(artwork.symmetry_lines[1].selected, false);
    assert_eq!(artwork.symmetry_lines[1].enabled, true);
    assert_eq!(artwork.symmetry_lines[1].origin_x, 48.0);
    assert_eq!(artwork.symmetry_lines[1].origin_y, 23.5);
    assert_eq!(artwork.symmetry_lines[1].angle, 1.6164448);
    assert_eq!(artwork.symmetry_lines[1].segment_count, 3);
    assert_eq!(
      artwork.symmetry_lines[1].symmetry_type,
      SymmetryType::Rotate
    );
  }

  fn get_frame_content_by_indices(artwork: &Artwork, i: usize, j: usize) -> &FrameContent {
    let frame_content_id = &artwork.layers[i].frames[j].content_id;

    artwork
      .frame_contents
      .iter()
      .find(|content| &content.id == frame_content_id)
      .unwrap()
  }
}
