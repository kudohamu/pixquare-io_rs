use crate::composite_type::{ArgbColor, Size};

pub(crate) fn wrapping_move_pixels_from_index(
  source: &[ArgbColor],
  target: &mut [ArgbColor],
  canvas_size: &Size,
  offset_x: i64,
  offset_y: i64,
) {
  for (source_index, pixel) in source.iter().enumerate() {
    let source_x = (source_index % canvas_size.width as usize) as i64;
    let source_y = (source_index / canvas_size.width as usize) as i64;

    let dest_x = (source_x + offset_x).rem_euclid(canvas_size.width as i64) as usize;
    let dest_y = (source_y + offset_y).rem_euclid(canvas_size.height as i64) as usize;
    let dest_index = dest_y * canvas_size.width as usize + dest_x;
    target[dest_index] = *pixel;
  }
}

pub(crate) fn move_pixel_from_index(
  source: &[ArgbColor],
  target: &mut [ArgbColor],
  source_index: usize,
  canvas_size: &Size,
  offset_x: i64,
  offset_y: i64,
) {
  let width = canvas_size.width as usize;
  let height = canvas_size.height as usize;
  let source_x = (source_index % width) as i64;
  let source_y = (source_index / width) as i64;

  let dest_x = source_x + offset_x as i64;
  let dest_y = source_y + offset_y as i64;

  if dest_x < 0 || dest_x > width as i64 - 1 || dest_y < 0 || dest_y > height as i64 - 1 {
    return;
  }

  let dest_index = dest_y as usize * width + dest_x as usize;
  target[dest_index] = source[source_index];
}

pub(crate) fn copy_pixel(
  source: &[ArgbColor],
  target: &mut [ArgbColor],
  canvas_size: &Size,
  source_x: usize,
  source_y: usize,
  dest_x: usize,
  dest_y: usize,
) {
  let source_index = source_y * canvas_size.width as usize + source_x;
  let dest_index = dest_y * canvas_size.width as usize + dest_x;

  if source_x > (canvas_size.width - 1) as usize || source_y > (canvas_size.height - 1) as usize {
    target[dest_index] = source[dest_index];
    return;
  }

  target[dest_index] = source[source_index];
}

pub(crate) fn smoothstep(d: f64, r_in: f64, r_out: f64) -> f64 {
  let t = ((d - r_in) / (r_out - r_in)).clamp(0., 1.);

  t * t * (3. - 2. * t)
}
