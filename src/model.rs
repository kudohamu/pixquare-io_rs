use half::f16;
use nom::{
  Parser,
  bytes::take,
  combinator::map_res,
  error::ErrorKind,
  number::complete::{le_f32, le_u8, le_u16, le_u32, le_u64},
};

use crate::{
  combinator::{
    argb_color, array_type, blend_mode, bool, compressed_colors, corners, dumb_string, float16,
    option_set_u8, size, string,
  },
  composite_type::{ArgbColor, BlendMode, Corners, CustomDataType, EntryType, FxType, Size},
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

/// Store some custom data set by the users.
/// https://docs.pixquare.art/pixquare-file/binary-specs#content
#[derive(Debug, Clone)]
pub enum CustomData {
  String(String),
}

impl CustomData {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = CustomDataHeader::parse(input)?;

    match header.data_type {
      CustomDataType::String => {
        let (input, str) = string(input)?;

        Ok((input, Self::String(str.to_string())))
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
  id: String,
  colors: Vec<ArgbColor>,
}

impl FrameContent {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = FrameContentHeader::parse(input)?;

    let (input, id) = dumb_string(header.id_len as usize).parse(input)?;
    let (input, colors) = compressed_colors(header.compressed_color_len as usize).parse(input)?;

    Ok((
      input,
      Self {
        id: id.to_string(),
        colors,
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
}

impl Frame {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = FrameHeader::parse(input)?;

    let (input, (id, duration, selected, content_id, opacity, z_index, custom_datas)) = (
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
      input,
      Self {
        id: id.to_string(),
        duration,
        selected,
        content_id: content_id.to_string(),
        opacity,
        z_index,
        custom_datas,
      },
    ))
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
}

impl FxColorOverlayContent {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    (bool, argb_color, blend_mode)
      .map(|(enabled, color, blend_mode)| Self {
        enabled,
        color,
        blend_mode,
      })
      .parse(input)
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
}

impl FxOutlineContent {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    (
      bool,
      corners,
      argb_color,
      array_type(argb_color),
      bool,
      bool,
    )
      .map(
        |(enabled, corners, color, ignored_colors, is_outside, is_water_color_on)| Self {
          enabled,
          corners,
          color,
          ignored_colors,
          is_outside,
          is_water_color_on,
        },
      )
      .parse(input)
  }
}

/// Content data of anti-aliasing for Fx.
#[derive(Debug, Clone)]
pub struct FxAntiAliasingContent {
  pub enabled: bool,
  /// Set of corners.
  pub corners: Vec<Corners>,
  pub intensity: f32,
}

impl FxAntiAliasingContent {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    (bool, array_type(corners), le_f32)
      .map(|(enabled, corners, intensity)| Self {
        enabled,
        corners,
        intensity,
      })
      .parse(input)
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
}

impl FxPatternOverlaryContent {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    (
      bool,
      array_type(le_u8).map_res(|data| data.chunks(4).map(|chunk| chunk.try_into()).collect()),
      size,
      le_f32,
      blend_mode,
    )
      .map(
        |(enabled, patterns, pattern_size, opacity, blend_mode)| Self {
          enabled,
          patterns,
          pattern_size,
          opacity,
          blend_mode,
        },
      )
      .parse(input)
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
        let (input, content) = FxColorOverlayContent::parse(input)?;

        Ok((input, Fx::ColorOverlay(content)))
      }
      FxType::Outline => {
        let (input, content) = FxOutlineContent::parse(input)?;

        Ok((input, Fx::Outline(content)))
      }
      FxType::AntiAliasing => {
        let (input, content) = FxAntiAliasingContent::parse(input)?;

        Ok((input, Fx::AntiAliasing(content)))
      }
      FxType::PatternOverlay => {
        let (input, content) = FxPatternOverlaryContent::parse(input)?;

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
}

impl Entry {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = EntryHeader::parse(input)?;

    let (rest, input) = take(header.data_size).parse(input)?;
    let (input, id_len) = le_u8(input)?;
    let (_input, id) = dumb_string(id_len as usize).parse(input)?;

    Ok((
      rest,
      Self {
        id_len,
        id: id.to_string(),
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
}

impl Group {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = GroupHeader::parse(input)?;

    let (rest, input) = take(header.data_size).parse(input)?;
    let (
      _input,
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
}

impl Layer {
  fn parse(input: &[u8]) -> PQResult<&[u8], Self> {
    let (input, header) = LayerHeader::parse(input)?;

    let (
      input,
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
      input,
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
      },
    ))
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

    let (input, (id, canvas_size, entries, groups, layers, frame_contents)) = (
      dumb_string(header.id_len as usize),
      size,
      array_type(Entry::parse),
      array_type(Group::parse),
      array_type(Layer::parse),
      array_type(FrameContent::parse),
    )
      .parse(input)?;

    Ok((
      input,
      Self {
        id: id.to_string(),
        canvas_size,
        entries,
        groups,
        layers,
        frame_contents,
      },
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn read_file_data() {
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
  }
}
