use half::f16;
use nom::{
  IResult, Parser,
  bytes::take,
  combinator::map_res,
  number::complete::{le_u8, le_u16, le_u32, le_u64},
};

use crate::{
  combinator::{argb_color, array_type, bool, dumb_string, float16, option_set_u8, size},
  composite_type::{ArgbColor, EntryType, Size},
  error::Error,
  primitive_type::OptionSet,
};

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
  fn parse(input: &[u8]) -> IResult<&[u8], Self> {
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
  fn parse(input: &[u8]) -> IResult<&[u8], Self> {
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
  fn parse(input: &[u8]) -> IResult<&[u8], Self> {
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
  fn parse(input: &[u8]) -> IResult<&[u8], Self> {
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
  fn parse(input: &[u8]) -> IResult<&[u8], Self> {
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
}

impl<'a> Artwork {
  pub fn read(buf: &'a [u8]) -> Result<Self, Error<'a>> {
    let result = Self::parse(&buf).map_err(|e| match e {
      nom::Err::Error(e) => e.into(),
      nom::Err::Incomplete(e) => e.into(),
      nom::Err::Failure(_) => Error::NomFailure,
    });

    match result {
      Ok((_rest, artwork)) => Ok(artwork),
      Err(e) => Err(e),
    }
  }

  fn parse(input: &[u8]) -> IResult<&[u8], Self> {
    let (input, header) = ArtworkHeader::parse(input)?;
    println!("{:?}", header);

    let (input, id) = dumb_string(header.id_len as usize).parse(input)?;
    let (input, canvas_size) = size(input)?;
    let (input, entries) = array_type(Entry::parse).parse(input)?;
    let (input, groups) = array_type(Group::parse).parse(input)?;

    Ok((
      input,
      Self {
        id: id.to_string(),
        canvas_size,
        entries,
        groups,
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
