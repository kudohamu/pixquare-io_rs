use std::{
  fmt::{Debug, Display},
  str::Utf8Error,
};

use nom::error::{ErrorKind, FromExternalError};

/// Represent error to parse .px file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError<I> {
  /// Invalid binary array or value of ArgbColor.
  InvalidArgbColorFormat,
  /// Unknown value of BlendMode.
  InvalidBlendMode,
  /// Unknown value of data type of CustomData
  InvalidCustomDataType,
  /// Unknown value of Fx Type.
  InvalidFxType,
  /// Unknown value of Entry Type.
  InvalidEntryType,
  /// Unknown value of GuideLine Type.
  InvalidGuideLineType,
  /// Unknown value of Processor Type.
  InvalidProcessorType,
  /// Unknown value of Modifier Type.
  InvalidModifierType,
  /// Unknown value of PaletteOrganization Type.
  InvalidPaletteOrganizationType,
  /// Unknown value of animation direction.
  InvalidAnimationDirection,
  /// Unknown value of color depth.
  InvalidColorDepth,
  /// Invalid UTF-8 binary.
  InvalidUtf8Error(Utf8Error),
  /// Failed to decompress zlib data.
  DecompressZlibError,
  Nom(I, ErrorKind),
}

impl<I> Display for ParseError<I> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidArgbColorFormat => write!(f, "invalid binary array or value of ArgbColor"),
      Self::InvalidBlendMode => write!(f, "unknown value of BlendMode"),
      Self::InvalidCustomDataType => write!(f, "unknown data type value of CustomData"),
      Self::InvalidFxType => write!(f, "unknown value of Fx Type"),
      Self::InvalidEntryType => write!(f, "unknown value of Entry Type"),
      Self::InvalidGuideLineType => write!(f, "unknown value of GuideLine Type"),
      Self::InvalidProcessorType => write!(f, "unknown value of Processor Type"),
      Self::InvalidModifierType => write!(f, "unknown value of Modifier Type"),
      Self::InvalidPaletteOrganizationType => {
        write!(f, "unknown value of PaletteOrganization Type")
      }
      Self::InvalidAnimationDirection => write!(f, "unknown value of animation direction"),
      Self::InvalidColorDepth => write!(f, "unknown value of color depth"),
      Self::InvalidUtf8Error(e) => write!(f, "invalid UTF-8 error: {e}"),
      Self::DecompressZlibError => write!(f, "failed to decompress zlib data"),
      Self::Nom(_, e) => write!(f, "nom error: {:?}", e),
    }
  }
}

impl<I: Debug> std::error::Error for ParseError<I> {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::InvalidUtf8Error(e) => Some(e),
      _ => None,
    }
  }
}

impl<I> From<Utf8Error> for ParseError<I> {
  fn from(e: Utf8Error) -> Self {
    Self::InvalidUtf8Error(e)
  }
}

impl<I, E> FromExternalError<I, E> for ParseError<I> {
  fn from_external_error(input: I, kind: ErrorKind, _err: E) -> Self {
    ParseError::Nom(input, kind)
  }
}

impl<I> nom::error::ParseError<I> for ParseError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    ParseError::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

impl<'a> From<ParseError<&'a str>> for nom::Err<ParseError<&'a str>> {
  fn from(err: ParseError<&'a str>) -> Self {
    nom::Err::Error(err)
  }
}

pub type PQResult<I, T> = nom::IResult<I, T, ParseError<I>>;
