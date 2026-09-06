use crate::{
  composite_type::BlendMode,
  model::{Frame, Layer, TilemapLayer},
};

#[derive(Debug, Clone)]
pub(crate) enum RenderFrameData {
  RegularLayer(RenderFrameDataRegularLayer),
  TilemapLayer(RenderFrameDataTilemapLayer),
}

#[derive(Debug, Clone)]
pub(crate) struct RenderFrameDataRegularLayer {
  pub frame: Frame,
  pub blend_mode: BlendMode,
}

#[derive(Debug, Clone)]
pub(crate) struct RenderFrameDataTilemapLayer {
  pub frame: Frame,
  pub tileset_id: String,
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
