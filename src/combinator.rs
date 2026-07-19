use std::{io::Read, str::from_utf8};

use flate2::read::ZlibDecoder;
use half::f16;
use nom::{
  Parser,
  bytes::complete::take,
  combinator::{map, map_res},
  multi::{count, many0},
  number::complete::{le_i32, le_u8, le_u16, le_u32, le_u64},
};

use crate::{
  composite_type::{ArgbColor, BlendMode, Coordinate, Corners, Rect, Size},
  error::{PPResult, ParseError},
  primitive_type::{DumbString, OptionSet},
};

/// Combinator for UTF8 data of a string.
pub fn dumb_string<'a>(
  len: usize,
) -> impl Parser<&'a [u8], Output = DumbString, Error = ParseError<&'a [u8]>> {
  map(map_res(take(len), from_utf8), |s| s.to_string().into())
}

/// Combinator(complete version) for UTF8 string of .px binary spec.
pub fn string(input: &[u8]) -> PPResult<&[u8], String> {
  let (input, ds) = le_u16
    .flat_map(|len| dumb_string(len as usize))
    .parse(input)?;

  Ok((input, ds.to_string()))
}

/// Combinator(complete version) for 16-bit float.
pub fn float16(input: &[u8]) -> PPResult<&[u8], f16> {
  le_u16.map(|bits| f16::from_bits(bits)).parse(input)
}

/// Combinator(complete version) for a boolean value, 1 byte.
pub fn bool(input: &[u8]) -> PPResult<&[u8], bool> {
  le_u8.map(|val| val != 0).parse(input)
}

/// Combinator(complete version) for OptionSet<UInt8>.
pub fn option_set_u8(input: &[u8]) -> PPResult<&[u8], OptionSet<u8>> {
  let (input, v) = le_u8(input)?;

  Ok((input, OptionSet::new(v)))
}

/// Combinator for n consecutive value of type.
pub fn type_n<I, F>(
  parser: F,
  n: usize,
) -> impl Parser<I, Output = Vec<<F as Parser<I>>::Output>, Error = <F as Parser<I>>::Error>
where
  I: Clone,
  F: Parser<I>,
{
  count(parser, n)
}

/// Combinator for array of type.
pub fn array_type<'a, F>(
  parser: F,
) -> impl Parser<
  &'a [u8],
  Output = Vec<<F as Parser<&'a [u8]>>::Output>,
  Error = <F as Parser<&'a [u8]>>::Error,
> + Clone
where
  F: Parser<&'a [u8]> + Clone,
{
  move |input: &'a [u8]| {
    let (input, len) = le_u64.parse(input)?;

    type_n(parser.clone(), len as usize).parse(input)
  }
}

/// Combinator(complete version) for coordinate.
pub fn coordinate(input: &[u8]) -> PPResult<&[u8], Coordinate> {
  let (input, (x, y)) = (le_i32, le_i32).parse(input)?;

  Ok((input, Coordinate { x, y }))
}

/// Combinator(complete version) for size.
pub fn size(input: &[u8]) -> PPResult<&[u8], Size> {
  let (input, (width, height)) = (le_u32, le_u32).parse(input)?;

  Ok((input, Size { width, height }))
}

/// Combinator(complete version) for rect.
pub fn rect(input: &[u8]) -> PPResult<&[u8], Rect> {
  let (input, (origin, size)) = (coordinate, size).parse(input)?;

  Ok((input, Rect { origin, size }))
}

/// Combinator(complete version) for ARGBColor.
pub fn argb_color(input: &[u8]) -> PPResult<&[u8], ArgbColor> {
  let (input, (r, g, b, a)) = (le_u8, le_u8, le_u8, le_u8).parse(input)?;

  Ok((input, ArgbColor { r, g, b, a }))
}

/// Combinator for compressed argb colors ([ArgbColor]).
pub fn compressed_colors<'a>(
  compressed_len: usize,
) -> impl Parser<&'a [u8], Output = Vec<ArgbColor>, Error = ParseError<&'a [u8]>> + Clone {
  move |input: &'a [u8]| {
    let (remaining_input, compressed_data) = take(compressed_len)(input)?;

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

    Ok((remaining_input, colors))
  }
}

/// Combinator(complete version) for corners.
pub fn corners(input: &[u8]) -> PPResult<&[u8], Corners> {
  let (input, option_set) = option_set_u8(input)?;

  Ok((input, option_set.into()))
}

/// Combinator(complete version) for BlendMode.
pub fn blend_mode(input: &[u8]) -> PPResult<&[u8], BlendMode> {
  map_res(le_u16, |v| v.try_into()).parse(input)
}
