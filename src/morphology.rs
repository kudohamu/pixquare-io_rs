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

  let mut dialeted = vec![ArgbColor::TRANSPARENT; target.len()];
  dilate(&bin, &mut dialeted, canvas_size, corners);

  debug_assert_eq!(bin.len(), target.len());

  for (index, b) in bin.iter().enumerate() {
    if dialeted[index].a == 0 {
      continue;
    }
    if b.a == 0 {
      target[index] = *color;
    } else {
      target[index] = ArgbColor::TRANSPARENT;
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

  let mut erosed = vec![ArgbColor::TRANSPARENT; target.len()];
  erose(&bin, &mut erosed, canvas_size, corners);

  debug_assert_eq!(bin.len(), target.len());

  for (index, e) in erosed.iter().enumerate() {
    if bin[index].a == 0 {
      continue;
    }
    if e.a == 0 {
      target[index] = *color;
    } else {
      target[index] = ArgbColor::TRANSPARENT;
    }
  }
}

pub fn erose(
  source: &[ArgbColor],
  target: &mut [ArgbColor],
  canvas_size: &Size,
  corners: &Corners,
) {
  let kernels = make_kernels(source, canvas_size, corners);

  for index in 0..source.len() {
    if kernels.iter().all(|kernel| kernel[index].a != 0) {
      target[index] = ArgbColor::new(0, 0, 0, 255);
    }
  }
}

pub fn dilate(
  source: &[ArgbColor],
  target: &mut [ArgbColor],
  canvas_size: &Size,
  corners: &Corners,
) {
  let kernels = make_kernels(source, canvas_size, corners);

  for index in 0..source.len() {
    if kernels.iter().any(|kernel| kernel[index].a != 0) {
      target[index] = ArgbColor::new(0, 0, 0, 255);
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

fn make_kernels(
  source: &[ArgbColor],
  canvas_size: &Size,
  corners: &Corners,
) -> Vec<Vec<ArgbColor>> {
  let mut kernels = CornersKernel::new(source.len(), corners);

  for (source_index, pixel) in source.iter().enumerate() {
    if pixel.a == 0 {
      continue;
    }

    let source_x = source_index % canvas_size.width as usize;
    let source_y = source_index / canvas_size.width as usize;

    if corners.top_left {
      shift(
        source,
        &mut kernels.top_left,
        canvas_size,
        source_x,
        source_y,
        -1,
        -1,
      );
    }
    if corners.top {
      shift(
        source,
        &mut kernels.top,
        canvas_size,
        source_x,
        source_y,
        0,
        -1,
      );
    }
    if corners.top_right {
      shift(
        source,
        &mut kernels.top_right,
        canvas_size,
        source_x,
        source_y,
        1,
        -1,
      );
    }
    if corners.right {
      shift(
        source,
        &mut kernels.right,
        canvas_size,
        source_x,
        source_y,
        1,
        0,
      );
    }
    if corners.bottom_right {
      shift(
        source,
        &mut kernels.bottom_right,
        canvas_size,
        source_x,
        source_y,
        1,
        1,
      );
    }
    if corners.bottom {
      shift(
        source,
        &mut kernels.bottom,
        canvas_size,
        source_x,
        source_y,
        0,
        1,
      );
    }
    if corners.bottom_left {
      shift(
        source,
        &mut kernels.bottom_left,
        canvas_size,
        source_x,
        source_y,
        -1,
        1,
      );
    }
    if corners.left {
      shift(
        source,
        &mut kernels.left,
        canvas_size,
        source_x,
        source_y,
        -1,
        0,
      );
    }
  }

  kernels.to_array()
}

fn shift(
  source: &[ArgbColor],
  target: &mut [ArgbColor],
  canvas_size: &Size,
  source_x: usize,
  source_y: usize,
  offset_x: i8,
  offset_y: i8,
) {
  let target_x = if offset_x > 0 {
    Some(source_x + offset_x as usize)
  } else {
    source_x.checked_sub(offset_x.abs() as usize)
  };
  let Some(target_x) = target_x else {
    return;
  };
  let target_y = if offset_y > 0 {
    Some(source_y + offset_y as usize)
  } else {
    source_y.checked_sub(offset_y.abs() as usize)
  };
  let Some(target_y) = target_y else {
    return;
  };
  if target_x > canvas_size.width as usize - 1 {
    return;
  }
  if target_y > canvas_size.height as usize - 1 {
    return;
  }

  let source_index = source_y * canvas_size.width as usize + source_x;
  let target_index = target_y * canvas_size.width as usize + target_x;
  target[target_index] = source[source_index];
}

#[derive(Debug, Clone, Default)]
struct CornersKernel {
  pub top_left: Vec<ArgbColor>,
  pub top: Vec<ArgbColor>,
  pub top_right: Vec<ArgbColor>,
  pub right: Vec<ArgbColor>,
  pub bottom_right: Vec<ArgbColor>,
  pub bottom: Vec<ArgbColor>,
  pub bottom_left: Vec<ArgbColor>,
  pub left: Vec<ArgbColor>,
  corners: Corners,
}

impl CornersKernel {
  fn new(len: usize, corners: &Corners) -> Self {
    Self {
      top_left: vec![ArgbColor::TRANSPARENT; len],
      top: vec![ArgbColor::TRANSPARENT; len],
      top_right: vec![ArgbColor::TRANSPARENT; len],
      right: vec![ArgbColor::TRANSPARENT; len],
      bottom_right: vec![ArgbColor::TRANSPARENT; len],
      bottom: vec![ArgbColor::TRANSPARENT; len],
      bottom_left: vec![ArgbColor::TRANSPARENT; len],
      left: vec![ArgbColor::TRANSPARENT; len],
      corners: *corners,
    }
  }

  fn to_array(&self) -> Vec<Vec<ArgbColor>> {
    let mut a = Vec::new();

    if self.corners.top_left {
      a.push(self.top_left.clone());
    }
    if self.corners.top {
      a.push(self.top.clone());
    }
    if self.corners.top_right {
      a.push(self.top_right.clone());
    }
    if self.corners.right {
      a.push(self.right.clone());
    }
    if self.corners.bottom_right {
      a.push(self.bottom_right.clone());
    }
    if self.corners.bottom {
      a.push(self.bottom.clone());
    }
    if self.corners.bottom_left {
      a.push(self.bottom_left.clone());
    }
    if self.corners.left {
      a.push(self.left.clone());
    }

    a
  }
}
