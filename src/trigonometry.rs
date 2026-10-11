pub(crate) fn safe_sin(angle: f32) -> f32 {
  let val = angle.sin();
  if val.abs() < 1e-7 { 0.0 } else { val }
}

pub(crate) fn safe_cos(angle: f32) -> f32 {
  let val = angle.cos();
  if val.abs() < 1e-7 { 0.0 } else { val }
}
