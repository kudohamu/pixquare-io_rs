use crate::composite_type::{ArgbColor, Corners, Size};

pub(crate) fn outside_gradient(
  source: &[ArgbColor],
  target: &mut [ArgbColor],
  canvas_size: &Size,
  corners: &Corners,
  color: &ArgbColor,
  ignore_colors: &[ArgbColor],
) {
  let mut bin = vec![ArgbColor::TRANSPARENT; target.len()];
  binarize(source, &mut bin, ignore_colors);
  debug_assert_eq!(bin.len(), target.len());

  for (index, target_pixel) in target.iter_mut().enumerate() {
    if bin[index].a == 0 && is_dilated_at(&bin, canvas_size, corners, index) {
      *target_pixel = *color;
    } else {
      *target_pixel = ArgbColor::TRANSPARENT;
    }
  }
}

pub(crate) fn inside_gradient(
  source: &[ArgbColor],
  target: &mut [ArgbColor],
  canvas_size: &Size,
  corners: &Corners,
  color: &ArgbColor,
  ignore_colors: &[ArgbColor],
) {
  let mut bin = vec![ArgbColor::TRANSPARENT; target.len()];
  binarize(source, &mut bin, ignore_colors);
  debug_assert_eq!(bin.len(), target.len());

  for (index, target_pixel) in target.iter_mut().enumerate() {
    if bin[index].a != 0 && !is_eroded_at(&bin, canvas_size, corners, index) {
      *target_pixel = *color;
    } else {
      *target_pixel = ArgbColor::TRANSPARENT;
    }
  }
}

pub(crate) fn hit_or_miss(
  source: &[ArgbColor],
  target: &mut [ArgbColor],
  canvas_size: &Size,
  corners: &[Corners],
  intensity: f32,
) {
  let mut bin = vec![ArgbColor::TRANSPARENT; target.len()];
  binarize(source, &mut bin, &[]);

  for (index, pixel) in bin.iter().enumerate() {
    if pixel.a != 0 {
      continue;
    }
    if corners.is_empty() {
      continue;
    }
    let coordinates: Vec<Vec<(usize, usize)>> = corners
      .iter()
      .filter_map(|corner| check_corner_all_black(&bin, canvas_size, index, corner))
      .collect();

    if coordinates.is_empty() {
      continue;
    }

    let mut coordinates: Vec<_> = coordinates.iter().flatten().collect();
    coordinates.sort_unstable();
    coordinates.dedup();

    let mut r = 0;
    let mut g = 0;
    let mut b = 0;
    let mut a = 0;

    for coordinate in &coordinates {
      let index = coordinate.1 * canvas_size.width as usize + coordinate.0;
      let color = source[index];

      r += color.r as usize;
      g += color.g as usize;
      b += color.b as usize;
      a += color.a as usize;
    }

    target[index] = ArgbColor::new(
      (r / coordinates.len()) as u8,
      (g / coordinates.len()) as u8,
      (b / coordinates.len()) as u8,
      (a / coordinates.len()) as u8,
    );
    target[index].multiply_alpha_f32(intensity);
  }
}

pub(crate) fn binarize(
  source: &[ArgbColor],
  target: &mut [ArgbColor],
  ignore_colors: &[ArgbColor],
) {
  for (index, pixel) in source.iter().enumerate() {
    if pixel.a != 0
      && ignore_colors
        .iter()
        .all(|ignore_color| pixel != ignore_color)
    {
      target[index] = ArgbColor::new(0, 0, 0, 255);
    }
  }
}

fn is_dilated_at(
  source: &[ArgbColor],
  canvas_size: &Size,
  corners: &Corners,
  target_index: usize,
) -> bool {
  get_offsets_from_corners(corners)
    .iter()
    .any(|(offset_x, offset_y)| {
      is_shift_source_black(source, canvas_size, target_index, *offset_x, *offset_y)
    })
}

fn is_eroded_at(
  source: &[ArgbColor],
  canvas_size: &Size,
  corners: &Corners,
  target_index: usize,
) -> bool {
  get_offsets_from_corners(corners)
    .iter()
    .all(|(offset_x, offset_y)| {
      is_shift_source_black(source, canvas_size, target_index, *offset_x, *offset_y)
    })
}

fn get_offsets_from_corners(corners: &Corners) -> Vec<(i8, i8)> {
  let mut offsets = Vec::new();

  if corners.top_left {
    offsets.push((-1, -1));
  }
  if corners.top {
    offsets.push((0, -1));
  }
  if corners.top_right {
    offsets.push((1, -1));
  }
  if corners.right {
    offsets.push((1, 0));
  }
  if corners.bottom_right {
    offsets.push((1, 1));
  }
  if corners.bottom {
    offsets.push((0, 1));
  }
  if corners.bottom_left {
    offsets.push((-1, 1));
  }
  if corners.left {
    offsets.push((-1, 0));
  }

  offsets
}

fn is_shift_source_black(
  source: &[ArgbColor],
  canvas_size: &Size,
  target_index: usize,
  offset_x: i8,
  offset_y: i8,
) -> bool {
  let width = canvas_size.width as usize;
  let height = canvas_size.height as usize;

  let target_x = target_index % width;
  let target_y = target_index / width;

  let source_x = if offset_x >= 0 {
    target_x.checked_sub(offset_x as usize)
  } else {
    Some(target_x + offset_x.unsigned_abs() as usize)
  };
  let Some(source_x) = source_x else {
    return false;
  };
  if source_x > width - 1 {
    return false;
  }

  let source_y = if offset_y >= 0 {
    target_y.checked_sub(offset_y as usize)
  } else {
    Some(target_y + offset_y.unsigned_abs() as usize)
  };
  let Some(source_y) = source_y else {
    return false;
  };
  if source_y > height - 1 {
    return false;
  }

  let source_index = source_y * width + source_x;

  source[source_index].a != 0
}

fn check_corner_all_black(
  source: &[ArgbColor],
  canvas_size: &Size,
  source_index: usize,
  corners: &Corners,
) -> Option<Vec<(usize, usize)>> {
  let width = canvas_size.width as usize;
  let height = canvas_size.height as usize;
  let source_x = source_index % width;
  let source_y = source_index / width;
  let offsets = get_offsets_from_corners(corners);

  offsets
    .into_iter()
    .map(|(offset_x, offset_y)| {
      let target_x = if offset_x >= 0 {
        Some(source_x + offset_x as usize)
      } else {
        source_x.checked_sub(offset_x.unsigned_abs() as usize)
      };
      let target_x = target_x?;
      if target_x > width - 1 {
        return None;
      }
      let target_y = if offset_y >= 0 {
        Some(source_y + offset_y as usize)
      } else {
        source_y.checked_sub(offset_y.unsigned_abs() as usize)
      };
      let target_y = target_y?;
      if target_y > height - 1 {
        return None;
      }

      let target_index = target_y * width + target_x;
      if source[target_index].a == 0 {
        return None;
      }

      Some((target_x, target_y))
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn assert_only_colored_at(pixels: &[ArgbColor], expected_indices: &[usize], color: ArgbColor) {
    let colored_indices = pixels
      .iter()
      .enumerate()
      .filter_map(|(index, pixel)| (pixel.a != 0).then_some(index))
      .collect::<Vec<_>>();

    assert_eq!(colored_indices, expected_indices);
    for &index in expected_indices {
      assert_eq!(pixels[index], color);
    }
  }

  #[test]
  fn test_outside_gradient_with_only_top() {
    let size = Size::new(5, 5);
    let mut source = vec![ArgbColor::TRANSPARENT; 25];
    source[6] = ArgbColor::new(1, 2, 3, 255);
    source[11] = ArgbColor::new(1, 2, 3, 255);
    let corners = Corners {
      top: true,
      ..Default::default()
    };
    let outline = ArgbColor::new(4, 5, 6, 255);
    let mut outside = vec![ArgbColor::TRANSPARENT; 25];

    outside_gradient(&source, &mut outside, &size, &corners, &outline, &[]);

    assert_only_colored_at(&outside, &[1], outline);
  }

  #[test]
  fn test_inside_gradient_with_only_top() {
    let size = Size::new(5, 5);
    let mut source = vec![ArgbColor::TRANSPARENT; 25];
    source[6] = ArgbColor::new(1, 2, 3, 255);
    source[11] = ArgbColor::new(1, 2, 3, 255);
    let corners = Corners {
      top: true,
      ..Default::default()
    };
    let outline = ArgbColor::new(4, 5, 6, 255);
    let mut inside = vec![ArgbColor::TRANSPARENT; 25];

    inside_gradient(&source, &mut inside, &size, &corners, &outline, &[]);

    assert_only_colored_at(&inside, &[11], outline);
  }

  #[test]
  fn test_outside_gradient_with_only_left() {
    let size = Size::new(5, 5);
    let mut source = vec![ArgbColor::TRANSPARENT; 25];
    source[6] = ArgbColor::new(1, 2, 3, 255);
    source[7] = ArgbColor::new(1, 2, 3, 255);
    let corners = Corners {
      left: true,
      ..Default::default()
    };
    let outline = ArgbColor::new(4, 5, 6, 255);
    let mut outside = vec![ArgbColor::TRANSPARENT; 25];

    outside_gradient(&source, &mut outside, &size, &corners, &outline, &[]);

    assert_only_colored_at(&outside, &[5], outline);
  }

  #[test]
  fn test_inside_gradient_with_only_left() {
    let size = Size::new(5, 5);
    let mut source = vec![ArgbColor::TRANSPARENT; 25];
    source[6] = ArgbColor::new(1, 2, 3, 255);
    source[7] = ArgbColor::new(1, 2, 3, 255);
    let corners = Corners {
      left: true,
      ..Default::default()
    };
    let outline = ArgbColor::new(4, 5, 6, 255);
    let mut inside = vec![ArgbColor::TRANSPARENT; 25];

    inside_gradient(&source, &mut inside, &size, &corners, &outline, &[]);

    assert_only_colored_at(&inside, &[7], outline);
  }

  #[test]
  fn test_gradients_with_only_diagonal_directions() {
    let size = Size::new(5, 5);
    let mut source = vec![ArgbColor::TRANSPARENT; 25];
    source[12] = ArgbColor::new(1, 2, 3, 255);
    let corners = Corners {
      top_left: true,
      top_right: true,
      bottom_right: true,
      bottom_left: true,
      ..Default::default()
    };
    let outline = ArgbColor::new(4, 5, 6, 255);
    let mut outside = vec![ArgbColor::TRANSPARENT; 25];
    let mut inside = vec![ArgbColor::TRANSPARENT; 25];

    outside_gradient(&source, &mut outside, &size, &corners, &outline, &[]);
    inside_gradient(&source, &mut inside, &size, &corners, &outline, &[]);

    assert_only_colored_at(&outside, &[6, 8, 16, 18], outline);
    assert_only_colored_at(&inside, &[12], outline);
  }

  #[test]
  fn test_gradients_at_canvas_edge_do_not_wrap() {
    let size = Size::new(5, 5);
    let mut source = vec![ArgbColor::TRANSPARENT; 25];
    source[0] = ArgbColor::new(1, 2, 3, 255);
    let corners = Corners {
      top_left: true,
      top: true,
      top_right: true,
      right: true,
      bottom_right: true,
      bottom: true,
      bottom_left: true,
      left: true,
    };
    let outline = ArgbColor::new(4, 5, 6, 255);
    let mut outside = vec![ArgbColor::TRANSPARENT; 25];
    let mut inside = vec![ArgbColor::TRANSPARENT; 25];

    outside_gradient(&source, &mut outside, &size, &corners, &outline, &[]);
    inside_gradient(&source, &mut inside, &size, &corners, &outline, &[]);

    assert_only_colored_at(&outside, &[1, 5, 6], outline);
    assert_only_colored_at(&inside, &[0], outline);
  }

  #[test]
  fn test_ignored_colors_are_treated_as_transparent() {
    let size = Size::new(5, 5);
    let source_color = ArgbColor::new(1, 2, 3, 255);
    let ignored_color = ArgbColor::new(7, 8, 9, 255);
    let mut source = vec![ArgbColor::TRANSPARENT; 25];
    source[11] = source_color;
    source[12] = ignored_color;
    let corners = Corners {
      left: true,
      right: true,
      ..Default::default()
    };
    let outline = ArgbColor::new(4, 5, 6, 255);
    let mut outside = vec![ArgbColor::TRANSPARENT; 25];
    let mut inside = vec![ArgbColor::TRANSPARENT; 25];

    outside_gradient(
      &source,
      &mut outside,
      &size,
      &corners,
      &outline,
      &[ignored_color],
    );
    inside_gradient(
      &source,
      &mut inside,
      &size,
      &corners,
      &outline,
      &[ignored_color],
    );

    assert_only_colored_at(&outside, &[10, 12], outline);
    assert_only_colored_at(&inside, &[11], outline);
  }

  #[test]
  fn test_hit_or_miss_combines_neighbor_colors_and_applies_intensity() {
    let size = Size::new(5, 5);
    let mut source = vec![ArgbColor::TRANSPARENT; 25];
    source[7] = ArgbColor::new(200, 40, 80, 255);
    source[11] = ArgbColor::new(40, 120, 200, 255);
    let corners = [Corners {
      top: true,
      left: true,
      ..Default::default()
    }];
    let mut target = vec![ArgbColor::TRANSPARENT; 25];

    hit_or_miss(&source, &mut target, &size, &corners, 0.5);

    assert_only_colored_at(&target, &[12], ArgbColor::new(60, 40, 70, 128));
  }

  #[test]
  fn test_hit_or_miss_requires_every_neighbor_in_a_pattern() {
    let size = Size::new(5, 5);
    let mut source = vec![ArgbColor::TRANSPARENT; 25];
    source[7] = ArgbColor::new(200, 40, 80, 255);
    let corners = [Corners {
      top: true,
      left: true,
      ..Default::default()
    }];
    let mut target = vec![ArgbColor::TRANSPARENT; 25];

    hit_or_miss(&source, &mut target, &size, &corners, 1.0);

    assert_only_colored_at(&target, &[], ArgbColor::TRANSPARENT);
  }

  #[test]
  fn test_hit_or_miss_accepts_any_matching_pattern() {
    let size = Size::new(5, 5);
    let mut source = vec![ArgbColor::TRANSPARENT; 25];
    source[7] = ArgbColor::new(200, 40, 80, 255);
    source[11] = ArgbColor::new(40, 120, 200, 255);
    let corners = [
      Corners {
        top: true,
        bottom: true,
        ..Default::default()
      },
      Corners {
        top: true,
        left: true,
        ..Default::default()
      },
    ];
    let mut target = vec![ArgbColor::TRANSPARENT; 25];

    hit_or_miss(&source, &mut target, &size, &corners, 1.0);

    assert_only_colored_at(&target, &[12], ArgbColor::new(120, 80, 140, 255));
  }

  #[test]
  fn test_hit_or_miss_deduplicates_neighbors_shared_by_matching_patterns() {
    let size = Size::new(5, 5);
    let mut source = vec![ArgbColor::TRANSPARENT; 25];
    source[6] = ArgbColor::new(0, 0, 240, 255);
    source[7] = ArgbColor::new(240, 0, 0, 255);
    source[11] = ArgbColor::new(0, 240, 0, 255);
    let corners = [
      Corners {
        top: true,
        left: true,
        ..Default::default()
      },
      Corners {
        top_left: true,
        top: true,
        ..Default::default()
      },
    ];
    let mut target = vec![ArgbColor::TRANSPARENT; 25];

    hit_or_miss(&source, &mut target, &size, &corners, 1.0);

    assert_only_colored_at(&target, &[12], ArgbColor::new(80, 80, 80, 255));
  }

  #[test]
  fn test_hit_or_miss_does_not_wrap_at_canvas_edge() {
    let size = Size::new(3, 3);
    let mut source = vec![ArgbColor::TRANSPARENT; 9];
    source[2] = ArgbColor::new(200, 40, 80, 255);
    let corners = [Corners {
      left: true,
      ..Default::default()
    }];
    let mut target = vec![ArgbColor::TRANSPARENT; 9];

    hit_or_miss(&source, &mut target, &size, &corners, 1.0);

    assert_only_colored_at(&target, &[], ArgbColor::TRANSPARENT);
  }
}
