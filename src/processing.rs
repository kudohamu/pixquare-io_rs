use crate::composite_type::{ArgbColor, Size};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(crate) struct BloomColor {
  pub r: f64,
  pub g: f64,
  pub b: f64,
}

pub(crate) fn create_bloom_source(source: &[ArgbColor], threshold: f64) -> Vec<BloomColor> {
  let mut target = vec![BloomColor::default(); source.len()];
  let threshold = threshold.clamp(0., 1.);

  for (index, pixel) in source.iter().enumerate() {
    if pixel.a == 0 {
      continue;
    }

    let alpha = f64::from(pixel.a);
    let r = f64::from(pixel.r) / alpha;
    let g = f64::from(pixel.g) / alpha;
    let b = f64::from(pixel.b) / alpha;
    let lightness = (r.max(g).max(b) + r.min(g).min(b)) / 2.;

    let brightness = ((lightness - 0.5) * 2.).clamp(0., 1.);
    let weight = if brightness <= threshold {
      0.
    } else if threshold >= 1. {
      0.
    } else {
      (brightness - threshold) / (1. - threshold)
    };

    target[index] = BloomColor {
      r: r * weight,
      g: g * weight,
      b: b * weight,
    };
  }

  target
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_create_bloom_source_ignores_dark_half_and_preserves_color() {
    let source = [
      ArgbColor::new(58, 68, 102, 255),
      ArgbColor::new(228, 59, 68, 255),
      ArgbColor::new(255, 255, 255, 255),
    ];

    let bloom = create_bloom_source(&source, 0.);

    assert_eq!(bloom[0], BloomColor::default());
    assert!(bloom[1].r > bloom[1].g);
    assert!(bloom[1].r > bloom[1].b);
    assert_eq!(
      bloom[2],
      BloomColor {
        r: 1.,
        g: 1.,
        b: 1.
      }
    );
  }

  #[test]
  fn test_gaussian_blur_with_zero_sigma_is_identity() {
    let source = [
      BloomColor {
        r: 1.,
        g: 0.,
        b: 0.,
      },
      BloomColor {
        r: 0.,
        g: 1.,
        b: 0.,
      },
    ];
    let canvas_size = Size {
      width: 2,
      height: 1,
    };

    let blurred = apply_gaussian_blur(&source, &canvas_size, 0.);

    assert_eq!(blurred, source);
  }

  #[test]
  fn test_gaussian_blur_preserves_color_channels() {
    let source = [
      BloomColor::default(),
      BloomColor {
        r: 1.,
        g: 0.,
        b: 0.,
      },
      BloomColor::default(),
    ];
    let canvas_size = Size {
      width: 3,
      height: 1,
    };

    let blurred = apply_gaussian_blur(&source, &canvas_size, 1.);

    assert!(blurred[0].r > 0.);
    assert!(blurred[1].r > blurred[0].r);
    assert_eq!(blurred[0].g, 0.);
    assert_eq!(blurred[0].b, 0.);
  }
}
