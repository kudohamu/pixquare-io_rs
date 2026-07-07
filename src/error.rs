use std::{fmt::Display, str::Utf8Error};

/// Represent error in pixquare-loader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error<'a> {
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
  /// Unknown value of Organization Type.
  InvalidOrganizationType,
  /// Invalid UTF-8 binary.
  InvalidUtf8Error(Utf8Error),
  /// This variant indicate Error from nom combinators.
  NomError(nom::error::Error<&'a [u8]>),
  /// This variant indicate Incomplete error from nom.
  NomIncomplete(nom::Needed),
  /// This variant indicate Failure error from nom.
  NomFailure,
}

impl Display for Error<'_> {
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
      Self::InvalidOrganizationType => write!(f, "unknown value of Organization Type"),
      Self::InvalidUtf8Error(e) => write!(f, "invalid UTF-8 error: {e}"),
      Self::NomError(e) => write!(f, "nom error: {:?}", e.code),
      Self::NomIncomplete(_) => write!(f, "nom incompelete error"),
      Self::NomFailure => write!(f, "nom failure error"),
    }
  }
}

impl std::error::Error for Error<'_> {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::InvalidUtf8Error(e) => Some(e),
      _ => None,
    }
  }
}

impl From<Utf8Error> for Error<'_> {
  fn from(e: Utf8Error) -> Self {
    Self::InvalidUtf8Error(e)
  }
}

impl<'a> From<nom::error::Error<&'a [u8]>> for Error<'a> {
  fn from(e: nom::error::Error<&'a [u8]>) -> Self {
    Self::NomError(e)
  }
}

impl<'a> From<nom::Needed> for Error<'a> {
  fn from(e: nom::Needed) -> Self {
    Self::NomIncomplete(e)
  }
}
