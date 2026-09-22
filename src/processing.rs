use msgw3c::Blend;

use crate::composite_type::{ArgbColor, Size};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(crate) struct BloomColor {
  pub r: f64,
  pub g: f64,
  pub b: f64,
  pub a: f64,
}

impl Blend for BloomColor {
  fn from_color(c: msgw3c::color::C) -> Self {
    let color = c.to_straight_alpha();

    Self {
      r: f64::from(color.r),
      g: f64::from(color.g),
      b: f64::from(color.b),
      a: f64::from(color.a),
    }
  }

  fn to_color(&self) -> msgw3c::color::C {
    msgw3c::color::C::from_straight_alpha(
      self.r as f32,
      self.g as f32,
      self.b as f32,
      self.a as f32,
    )
  }
}

pub(crate) fn create_bloom_source(source: &[ArgbColor], intensity: f64) -> Vec<BloomColor> {
  let mut target = vec![BloomColor::default(); source.len()];

  for (index, pixel) in source.iter().enumerate() {
    if pixel.a == 0 {
      continue;
    }

    let alpha = f64::from(pixel.a);
    let r = f64::from(pixel.r) / alpha;
    let g = f64::from(pixel.g) / alpha;
    let b = f64::from(pixel.b) / alpha;
    let lightness = (r.max(g).max(b) + r.min(g).min(b)) / 2.;

    target[index] = BloomColor {
      r: r * lightness,
      g: g * lightness,
      b: b * lightness,
      a: intensity,
    };
  }

  target
}

pub(crate) fn create_bloom_difference(
  source: &[BloomColor],
  blurred: &[BloomColor],
  threshold: f64,
) -> Vec<BloomColor> {
  blurred
    .iter()
    .zip(source)
    .map(|(blurred, source)| BloomColor {
      r: apply_bloom_threshold((blurred.r - source.r).max(0.), threshold),
      g: apply_bloom_threshold((blurred.g - source.g).max(0.), threshold),
      b: apply_bloom_threshold((blurred.b - source.b).max(0.), threshold),
      a: source.a,
    })
    .collect()
}

fn apply_bloom_threshold(value: f64, threshold: f64) -> f64 {
  if value <= threshold {
    0.
  } else if threshold >= 1. {
    0.
  } else {
    (value - threshold) / (1. - threshold)
  }
}

pub(crate) fn apply_gaussian_blur(
  source: &[BloomColor],
  canvas_size: &Size,
  sigma: f64,
) -> Vec<BloomColor> {
  let width = canvas_size.width as usize;
  let height = canvas_size.height as usize;

  let mut target = vec![BloomColor::default(); source.len()];
  let (kernel_radius, kernel) = create_gaussian_kernel(sigma);
  let kernel_size = (kernel_radius * 2 + 1) as usize;

  for y in 0..height {
    for x in 0..width {
      let mut color = BloomColor::default();

      for (kernel_index, kernel_value) in kernel.iter().enumerate() {
        let kernel_x = kernel_index % kernel_size;
        let kernel_y = kernel_index / kernel_size;
        let offset_x = kernel_x as isize - kernel_radius;
        let offset_y = kernel_y as isize - kernel_radius;

        let source_x = (x as isize + offset_x).clamp(0, width as isize - 1) as usize;
        let source_y = (y as isize + offset_y).clamp(0, height as isize - 1) as usize;

        let source_index = source_y * width + source_x;
        color.r += source[source_index].r * kernel_value;
        color.g += source[source_index].g * kernel_value;
        color.b += source[source_index].b * kernel_value;
      }

      let target_index = y * width + x;
      target[target_index] = color;
    }
  }

  target
}

pub(crate) fn adjust_threshold(threshold: f64) -> f64 {
  let min = 0.;
  let max = 0.4;

  min + threshold / 1. * (max - min)
}

pub(crate) fn adjust_intensity(intensity: f64) -> f64 {
  0.7 + 0.7 * intensity
}

fn create_gaussian_kernel(sigma: f64) -> (isize, Vec<f64>) {
  if sigma <= 0. {
    return (0, vec![1.]);
  }

  let radius = (sigma * 3.).ceil() as usize;
  let width = radius * 2 + 1;
  let height = radius * 2 + 1;
  let kernel_size = width * height;
  let mut kernel = Vec::with_capacity(kernel_size);

  for y in 0..height {
    for x in 0..width {
      let offset_x = x as f64 - radius as f64;
      let offset_y = y as f64 - radius as f64;
      let weight = (-((offset_x * offset_x + offset_y * offset_y) / (2. * sigma * sigma))).exp();

      kernel.push(weight);
    }
  }

  let total: f64 = kernel.iter().sum();

  for weight in &mut kernel {
    *weight = *weight / total;
  }

  (radius as isize, kernel)
}
