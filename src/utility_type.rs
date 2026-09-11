use half::f16;
use msgw3c::{batch_blend_with, batch_normal, composite::PorterDuff};

use crate::{
  composite_type::{ArgbColor, EntryType, Size},
  error::ArtworkOperationError,
  model::{Artwork, Entry, Frame, FrameContent, Layer, TilemapFrameContent, TilemapLayer, Tileset},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerVisibility {
  Visible,
  All,
}

#[derive(Debug, Clone)]
pub(crate) enum RenderPlan<'a> {
  Group {
    opacity: f32,
    base_index: usize,
    children: Vec<RenderPlan<'a>>,
  },
  RegularLayer {
    layer: &'a Layer,
    frame: &'a Frame,
    content: &'a FrameContent,
    base_index: usize,
  },
  TilemapLayer {
    layer: &'a TilemapLayer,
    frame: &'a Frame,
    content: &'a TilemapFrameContent,
    tileset: &'a Tileset,
    canvas_size: Size,
    base_index: usize,
  },
}

impl<'a> RenderPlan<'a> {
  pub fn build(
    artwork: &'a Artwork,
    frame_index: usize,
    visibility: LayerVisibility,
  ) -> Result<Self, ArtworkOperationError> {
    let plans = Self::build_children(&artwork.entries, artwork, frame_index, visibility)?;

    Ok(Self::Group {
      opacity: 1.,
      base_index: 0,
      children: plans,
    })
  }

  fn build_children(
    entries: &'a Vec<Entry>,
    artwork: &'a Artwork,
    frame_index: usize,
    visibility: LayerVisibility,
  ) -> Result<Vec<RenderPlan<'a>>, ArtworkOperationError> {
    let mut plans = Vec::new();

    for (base_index, entry) in entries.iter().enumerate() {
      match entry.entry_type {
        EntryType::Group => {
          let group = artwork
            .groups
            .iter()
            .find(|group| group.id == entry.id.clone())
            .ok_or(ArtworkOperationError::GroupNotFound(entry.id.clone()))?;

          if visibility == LayerVisibility::Visible && !group.visible {
            continue;
          }

          let mut child_plans =
            Self::build_children(&group.child_entries, artwork, frame_index, visibility)?;
          child_plans.sort_by(|a, b| a.order_index().cmp(&b.order_index()));

          let plan = RenderPlan::Group {
            opacity: group.opacity.into(),
            base_index,
            children: child_plans,
          };
          plans.push(plan);
        }
        EntryType::RegularLayer => {
          let layer = artwork
            .layers
            .iter()
            .find(|layer| layer.id == entry.id)
            .ok_or(ArtworkOperationError::LayerNotFound(entry.id.clone()))?;

          if visibility == LayerVisibility::Visible && !layer.visible {
            continue;
          }

          let frame =
            layer
              .frames
              .get(frame_index)
              .ok_or(ArtworkOperationError::FrameIndexOutOfBounds((
                frame_index,
                layer.frames.len(),
              )))?;
          let content = artwork
            .frame_contents
            .iter()
            .find(|content| content.id == frame.content_id)
            .ok_or(ArtworkOperationError::FrameContentNotFound(
              frame.content_id.clone(),
            ))?;

          let plan = RenderPlan::RegularLayer {
            layer,
            frame: &frame,
            content,
            base_index,
          };
          plans.push(plan);
        }
        EntryType::TilemapLayer => {
          let layer = artwork
            .tilemap_layers
            .iter()
            .find(|layer| layer.id == entry.id)
            .ok_or(ArtworkOperationError::TilemapLayerNotFound(
              entry.id.clone(),
            ))?;

          if visibility == LayerVisibility::Visible && !layer.visible {
            continue;
          }

          let frame =
            layer
              .frames
              .get(frame_index)
              .ok_or(ArtworkOperationError::FrameIndexOutOfBounds((
                frame_index,
                layer.frames.len(),
              )))?;
          let content = artwork
            .tilemap_frame_contents
            .iter()
            .find(|content| content.id == frame.content_id)
            .ok_or(ArtworkOperationError::FrameContentNotFound(
              frame.content_id.clone(),
            ))?;
          let tileset = artwork
            .tilesets
            .iter()
            .find(|tileset| tileset.id == layer.tileset_id)
            .ok_or(ArtworkOperationError::TilesetNotFound(
              layer.tileset_id.clone(),
            ))?;

          let plan = RenderPlan::TilemapLayer {
            layer,
            frame,
            content,
            tileset,
            base_index,
            canvas_size: artwork.canvas_size,
          };
          plans.push(plan);
        }
        _ => (),
      }
    }

    plans.sort_by(|a, b| a.order_index().cmp(&b.order_index()));
    Ok(plans)
  }

  pub fn render_onto(
    &self,
    backdrop: &[ArgbColor],
  ) -> Result<Vec<ArgbColor>, ArtworkOperationError> {
    match self {
      Self::Group {
        opacity,
        base_index: _base_index,
        children,
      } => {
        let mut offscreen = vec![ArgbColor::TRANSPARENT; backdrop.len()];

        for plan in children {
          offscreen = plan.render_onto(&offscreen[..])?;
        }

        for color in offscreen.iter_mut() {
          color.multiply_alpha(*opacity);
        }

        let mut composited = vec![ArgbColor::TRANSPARENT; backdrop.len()];
        batch_normal(&offscreen, backdrop, &mut composited)?;

        Ok(composited)
      }
      Self::RegularLayer {
        layer,
        frame,
        content,
        base_index: _base_index,
      } => {
        let mut foreground = content.colors.clone();

        if frame.opacity == f16::from_f32(2.0) {
          foreground.multiply_alpha(layer.opacity.to_f32());
        } else {
          foreground.multiply_alpha(frame.opacity.to_f32());
        }

        let mut composited = vec![ArgbColor::TRANSPARENT; backdrop.len()];
        batch_blend_with(
          &foreground,
          &backdrop,
          &mut composited,
          layer.blend_mode.into(),
          PorterDuff::SourceOver,
        )?;

        Ok(composited)
      }
      Self::TilemapLayer {
        layer,
        frame,
        content,
        tileset,
        base_index: _base_index,
        canvas_size,
      } => {
        let canvas_pixel_len = (canvas_size.width * canvas_size.height) as usize;
        let mut composited = vec![ArgbColor::TRANSPARENT; canvas_pixel_len];
        let mut frame_data = vec![ArgbColor::TRANSPARENT; canvas_pixel_len];
        let grid_width = (canvas_size.width / content.tile_size.width) as usize;

        for grid_index in 0..content.tiles.len() {
          let tile_index = content.tiles[grid_index];

          if TilemapFrameContent::is_unassigned_tile(tile_index) {
            continue;
          }

          let grid_x = grid_index % grid_width;
          let grid_y = grid_index / grid_width;

          let tile_data = &tileset.tile_images[tile_index as usize];

          for tile_pixel_index in 0..tile_data.len() {
            let tile_x = tile_pixel_index % content.tile_size.width as usize;
            let tile_y = tile_pixel_index / content.tile_size.width as usize;

            let canvas_x = grid_x * content.tile_size.width as usize + tile_x;
            let canvas_y = grid_y * content.tile_size.height as usize + tile_y;
            let canvas_index = canvas_y * canvas_size.width as usize + canvas_x;

            frame_data[canvas_index] = tile_data[tile_pixel_index];
          }
        }

        if frame.opacity == f16::from_f32(2.0) {
          frame_data.multiply_alpha(layer.opacity.to_f32());
        } else {
          frame_data.multiply_alpha(frame.opacity.to_f32());
        }

        batch_blend_with(
          &frame_data,
          &backdrop,
          &mut composited,
          layer.blend_mode.into(),
          PorterDuff::SourceOver,
        )?;

        Ok(composited)
      }
    }
  }

  fn order_index(&self) -> i16 {
    match self {
      Self::Group {
        opacity: _opacity,
        base_index,
        children: _children,
      } => (*base_index) as i16,
      Self::RegularLayer {
        layer: _layer,
        frame,
        content: _content,
        base_index,
      } => {
        if frame.z_index == 0 {
          (*base_index) as i16
        } else {
          frame.z_index
        }
      }
      Self::TilemapLayer {
        layer: _layer,
        frame,
        content: _content,
        tileset: _tileset,
        base_index,
        canvas_size: _canvas_size,
      } => {
        if frame.z_index == 0 {
          (*base_index) as i16
        } else {
          frame.z_index
        }
      }
    }
  }
}

trait ArgbColorSlice {
  fn multiply_alpha(&mut self, alpha: f32);
}

impl ArgbColorSlice for [ArgbColor] {
  fn multiply_alpha(&mut self, alpha: f32) {
    for color in self {
      color.multiply_alpha(alpha);
    }
  }
}
