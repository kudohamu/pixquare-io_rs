use crate::composite_type::{ArgbColor, Corners, Size};

pub fn outside_gradient(
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

pub fn inside_gradient(
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

pub fn binarize(source: &[ArgbColor], target: &mut [ArgbColor], ignore_colors: &[ArgbColor]) {
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
    Some(target_x + offset_x.abs() as usize)
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
    Some(target_y + offset_y.abs() as usize)
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
}
