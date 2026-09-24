use half::f16;
use msgw3c::{batch_blend_with_in_place, batch_normal_in_place, composite::PorterDuff};

use crate::{
  composite_type::{ArgbColor, BlendMode, EntryType, Size},
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
    has_cropping_masks: bool,
    clipping_masks: Vec<RenderPlan<'a>>,
    cropping_masks: Vec<RenderPlan<'a>>,
    children: Vec<RenderPlan<'a>>,
  },
  RegularLayer {
    layer: &'a Layer,
    frame: &'a Frame,
    content: &'a FrameContent,
    base_index: usize,
    has_clipping_masks: bool,
    has_cropping_masks: bool,
    clipping_masks: Vec<RenderPlan<'a>>,
    cropping_masks: Vec<RenderPlan<'a>>,
    canvas_size: Size,
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
  pub(crate) fn build(
    artwork: &'a Artwork,
    frame_index: usize,
    visibility: LayerVisibility,
  ) -> Result<Self, ArtworkOperationError> {
    let plans = Self::build_children(&artwork.entries, artwork, frame_index, visibility)?;

    Ok(Self::Group {
      opacity: 1.,
      base_index: 0,
      has_cropping_masks: false,
      clipping_masks: Vec::new(),
      cropping_masks: Vec::new(),
      children: plans,
    })
  }

  fn build_children(
    entries: &'a [Entry],
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
          child_plans.sort_by(|a, b| {
            a.primary_order_index()
              .cmp(&b.primary_order_index())
              .then(a.secondary_order_index().cmp(&b.secondary_order_index()))
          });

          let mut clipping_masks =
            Self::build_children(&group.clipping_masks, artwork, frame_index, visibility)?;
          clipping_masks.sort_by(|a, b| {
            a.primary_order_index()
              .cmp(&b.primary_order_index())
              .then(a.secondary_order_index().cmp(&b.secondary_order_index()))
          });

          let mut cropping_masks =
            Self::build_children(&group.cropping_masks, artwork, frame_index, visibility)?;
          cropping_masks.sort_by(|a, b| {
            a.primary_order_index()
              .cmp(&b.primary_order_index())
              .then(a.secondary_order_index().cmp(&b.secondary_order_index()))
          });

          let plan = RenderPlan::Group {
            opacity: group.opacity.into(),
            base_index,
            has_cropping_masks: !group.cropping_masks.is_empty(),
            clipping_masks,
            cropping_masks,
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

          let mut clipping_masks =
            Self::build_children(&layer.clipping_masks, artwork, frame_index, visibility)?;
          clipping_masks.sort_by(|a, b| {
            a.primary_order_index()
              .cmp(&b.primary_order_index())
              .then(a.secondary_order_index().cmp(&b.secondary_order_index()))
          });

          let mut cropping_masks =
            Self::build_children(&layer.cropping_masks, artwork, frame_index, visibility)?;
          cropping_masks.sort_by(|a, b| {
            a.primary_order_index()
              .cmp(&b.primary_order_index())
              .then(a.secondary_order_index().cmp(&b.secondary_order_index()))
          });

          let plan = RenderPlan::RegularLayer {
            layer,
            frame,
            content,
            base_index,
            has_clipping_masks: !layer.clipping_masks.is_empty(),
            has_cropping_masks: !layer.cropping_masks.is_empty(),
            clipping_masks,
            cropping_masks,
            canvas_size: artwork.canvas_size,
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

    plans.sort_by(|a, b| {
      a.primary_order_index()
        .cmp(&b.primary_order_index())
        .then(a.secondary_order_index().cmp(&b.secondary_order_index()))
    });
    Ok(plans)
  }

  pub(crate) fn render_onto(
    &self,
    backdrop: &mut [ArgbColor],
  ) -> Result<(), ArtworkOperationError> {
    match self {
      Self::Group {
        opacity,
        has_cropping_masks,
        clipping_masks,
        cropping_masks,
        children,
        ..
      } => {
        let mut offscreen = vec![ArgbColor::TRANSPARENT; backdrop.len()];

        for plan in children {
          plan.render_onto(&mut offscreen)?;
        }

        for color in offscreen.iter_mut() {
          color.multiply_alpha_f32(*opacity);
        }

        if *has_cropping_masks {
          apply_cropping(&mut offscreen, cropping_masks)?;
        } else {
          for mask in clipping_masks {
            mask.clip_onto(&mut offscreen)?;
          }
        }

        batch_normal_in_place(&offscreen, backdrop)?;

        Ok(())
      }
      Self::RegularLayer {
        layer,
        frame,
        content,
        has_cropping_masks,
        clipping_masks,
        cropping_masks,
        canvas_size,
        ..
      } => {
        let Some(image) = &content._image else {
          return Ok(());
        };
        let mut foreground = image.clone();

        for fx in &layer.fxs {
          foreground = fx.apply(&foreground, canvas_size)?;
        }

        if *has_cropping_masks {
          apply_cropping(&mut foreground, cropping_masks)?;
        } else {
          for mask in clipping_masks {
            mask.clip_onto(&mut foreground)?;
          }
        }

        if frame.opacity == f16::from_f32(2.0) {
          foreground.multiply_alpha_f32(layer.opacity.to_f32());
        } else {
          foreground.multiply_alpha_f32(frame.opacity.to_f32());
        }

        batch_blend_with_in_place(
          &foreground,
          backdrop,
          self.effective_blend_mode().into(),
          PorterDuff::SourceOver,
        )?;

        Ok(())
      }
      Self::TilemapLayer {
        layer,
        frame,
        content,
        tileset,
        canvas_size,
        ..
      } => {
        let Some(tile_images) = &tileset._tile_images else {
          return Ok(());
        };

        let canvas_pixel_len = (canvas_size.width * canvas_size.height) as usize;
        let mut frame_data = vec![ArgbColor::TRANSPARENT; canvas_pixel_len];
        let grid_width = (canvas_size.width / content.tile_size.width) as usize;

        for grid_index in 0..content.tiles.len() {
          let tile_index = content.tiles[grid_index];

          if TilemapFrameContent::is_unassigned_tile(tile_index) {
            continue;
          }

          let grid_x = grid_index % grid_width;
          let grid_y = grid_index / grid_width;

          let tile_data = &tile_images[tile_index as usize];

          for (tile_pixel_index, tile_color) in tile_data.iter().enumerate() {
            let tile_x = tile_pixel_index % content.tile_size.width as usize;
            let tile_y = tile_pixel_index / content.tile_size.width as usize;

            let canvas_x = grid_x * content.tile_size.width as usize + tile_x;
            let canvas_y = grid_y * content.tile_size.height as usize + tile_y;
            let canvas_index = canvas_y * canvas_size.width as usize + canvas_x;

            frame_data[canvas_index] = *tile_color;
          }
        }

        if frame.opacity == f16::from_f32(2.0) {
          frame_data.multiply_alpha_f32(layer.opacity.to_f32());
        } else {
          frame_data.multiply_alpha_f32(frame.opacity.to_f32());
        }

        batch_blend_with_in_place(
          &frame_data,
          backdrop,
          layer.blend_mode.into(),
          PorterDuff::SourceOver,
        )?;

        Ok(())
      }
    }
  }

  fn primary_order_index(&self) -> i16 {
    match self {
      Self::Group { .. } => 0,
      Self::RegularLayer { frame, .. } => {
        if frame.z_index == 0 {
          0
        } else {
          frame.z_index
        }
      }
      Self::TilemapLayer { frame, .. } => {
        if frame.z_index == 0 {
          0
        } else {
          frame.z_index
        }
      }
    }
  }

  fn secondary_order_index(&self) -> i16 {
    match self {
      Self::Group { base_index, .. } => *base_index as i16,
      Self::RegularLayer { base_index, .. } => *base_index as i16,
      Self::TilemapLayer { base_index, .. } => *base_index as i16,
    }
  }

  fn clip_onto(&self, backdrop: &mut [ArgbColor]) -> Result<(), ArtworkOperationError> {
    let mut offscreen = vec![ArgbColor::TRANSPARENT; backdrop.len()];

    self.render_onto(&mut offscreen)?;

    for (index, m) in offscreen.iter_mut().enumerate() {
      let b = backdrop[index];

      m.multiply_alpha(b.a);
    }

    batch_blend_with_in_place(
      &offscreen,
      backdrop,
      self.effective_blend_mode().into(),
      PorterDuff::SourceOver,
    )?;

    Ok(())
  }

  fn effective_blend_mode(&self) -> BlendMode {
    match self {
      Self::RegularLayer {
        layer,
        has_clipping_masks,
        has_cropping_masks,
        ..
      } => {
        if *has_clipping_masks || *has_cropping_masks {
          BlendMode::Normal
        } else {
          layer.blend_mode
        }
      }
      Self::TilemapLayer { layer, .. } => layer.blend_mode,
      Self::Group { .. } => BlendMode::Normal,
    }
  }
}

fn apply_cropping<'a>(
  foreground: &mut [ArgbColor],
  cropping_masks: &Vec<RenderPlan<'a>>,
) -> Result<(), ArtworkOperationError> {
  let mut offscreen = vec![ArgbColor::TRANSPARENT; foreground.len()];
  for mask in cropping_masks {
    mask.render_onto(&mut offscreen)?;
  }
  for (index, m) in offscreen.iter().enumerate() {
    foreground[index].multiply_alpha(m.a);
  }

  Ok(())
}

trait ArgbColorSlice {
  fn multiply_alpha_f32(&mut self, alpha: f32);
}

impl ArgbColorSlice for [ArgbColor] {
  fn multiply_alpha_f32(&mut self, alpha: f32) {
    for color in self {
      color.multiply_alpha_f32(alpha);
    }
  }
}
