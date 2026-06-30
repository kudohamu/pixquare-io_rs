use std::str::{Utf8Error, from_utf8};

use nom::{
  IResult, Parser,
  bytes::take,
  combinator::map_res,
  error::{FromExternalError, ParseError},
  multi::count,
  number::complete::{le_u16, le_u64},
};

/// Combinator for UTF8 data of a string.
pub fn dumb_string<'a, E>(len: usize) -> impl Parser<&'a [u8], Output = &'a str, Error = E>
where
  E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], Utf8Error>,
{
  map_res(take(len), |bytes| from_utf8(bytes))
}

/// Combinator(Complete version) for UTF8 string of .px binary spec.
pub fn string(input: &[u8]) -> IResult<&[u8], &str> {
  le_u16
    .flat_map(|len| dumb_string(len as usize))
    .parse(input)
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
