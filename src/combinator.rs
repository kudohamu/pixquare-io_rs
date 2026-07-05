use std::str::from_utf8;

use half::f16;
use nom::{
  IResult, Parser,
  bytes::complete::take,
  combinator::map_res,
  error::Error,
  multi::count,
  number::complete::{le_i32, le_u8, le_u16, le_u32, le_u64},
};

use crate::{
  composite_type::{ArgbColor, BlendMode, Coordinate, Corners, Rect, Size},
  primitive_type::OptionSet,
};

/// Combinator for UTF8 data of a string.
pub fn dumb_string<'a>(
  len: usize,
) -> impl Parser<&'a [u8], Output = &'a str, Error = Error<&'a [u8]>> {
  map_res(take(len), |bytes| from_utf8(bytes))
}

/// Combinator(complete version) for UTF8 string of .px binary spec.
pub fn string(input: &[u8]) -> IResult<&[u8], &str> {
  le_u16
    .flat_map(|len| dumb_string(len as usize))
    .parse(input)
}

/// Combinator(complete version) for 16-bit float.
pub fn float16(input: &[u8]) -> IResult<&[u8], f16> {
  le_u16.map(|bits| f16::from_bits(bits)).parse(input)
}

/// Combinator(complete version) for a boolean value, 1 byte.
pub fn bool(input: &[u8]) -> IResult<&[u8], bool> {
  le_u8.map(|val| val != 0).parse(input)
}

/// Combinator(complete version) for OptionSet<UInt8>.
pub fn option_set_u8(input: &[u8]) -> IResult<&[u8], OptionSet<u8>> {
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
>
where
  F: Parser<&'a [u8]> + Clone,
{
  le_u64.flat_map(move |len| type_n(parser.clone(), len as usize))
}

/// Combinator(complete version) for coordinate.
pub fn coordinate(input: &[u8]) -> IResult<&[u8], Coordinate> {
  let (input, (x, y)) = (le_i32, le_i32).parse(input)?;

  Ok((input, Coordinate { x, y }))
}

/// Combinator(complete version) for size.
pub fn size(input: &[u8]) -> IResult<&[u8], Size> {
  let (input, (width, height)) = (le_u32, le_u32).parse(input)?;

  Ok((input, Size { width, height }))
}

/// Combinator(complete version) for rect.
pub fn rect(input: &[u8]) -> IResult<&[u8], Rect> {
  let (input, (origin, size)) = (coordinate, size).parse(input)?;

  Ok((input, Rect { origin, size }))
}

/// Combinator(complete version) for ARGBColor.
pub fn argb_color(input: &[u8]) -> IResult<&[u8], ArgbColor> {
  let (input, (r, g, b, a)) = (le_u8, le_u8, le_u8, le_u8).parse(input)?;

  Ok((input, ArgbColor { r, g, b, a }))
}

/// Combinator(complete version) for corners.
pub fn corners(input: &[u8]) -> IResult<&[u8], Corners> {
  let (input, option_set) = option_set_u8(input)?;

  Ok((input, option_set.into()))
}

/// Combinator(complete version) for BlendMode.
pub fn blend_mode(input: &[u8]) -> IResult<&[u8], BlendMode> {
  map_res(le_u16, |v| v.try_into()).parse(input)
}

#[cfg(test)]
mod tests {
  use nom::{Parser, error::ErrorKind};

  use crate::combinator::dumb_string;

  #[test]
  fn test_dumb_string() {
    assert_eq!(
      dumb_string(4).parse(b"RustRemaining"),
      Ok((&b"Remaining"[..], "Rust"))
    );
    assert_eq!(
      dumb_string(0).parse(b"RustRemaining"),
      Ok((&b"RustRemaining"[..], ""))
    );
    assert_eq!(
      dumb_string(4).parse(b"Rus"),
      Err(nom::Err::Error(nom::error::Error::new(
        &b"Rus"[..],
        ErrorKind::Eof,
      )))
    );
    assert_eq!(
      dumb_string(4).parse(b"\xff\xff\xff\xffRemaining"),
      Err(nom::Err::Error(nom::error::Error::new(
        &b"\xff\xff\xff\xffRemaining"[..],
        ErrorKind::MapRes
      )))
    )
  }
}
