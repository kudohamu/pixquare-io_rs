/// Represent error in pixquare-loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
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
}
