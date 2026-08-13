use crate::{
  composite_type::{BlendMode, EntryType},
  model::{Frame, Layer, TilemapLayer},
};

#[derive(Debug, Clone)]
pub(crate) struct RenderFrameData {
  pub entry_type: EntryType,
  pub frame: Frame,
  pub blend_mode: BlendMode,
}

#[derive(Debug, Clone)]
pub(crate) enum Layerable<'a> {
  RegularLayer(&'a Layer),
  TilemapLayer(&'a TilemapLayer),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerVisibility {
  Visible,
  All,
}
