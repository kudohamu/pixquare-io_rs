pub(crate) fn smoothstep(d: f64, r_in: f64, r_out: f64) -> f64 {
  let t = ((d - r_in) / (r_out - r_in)).clamp(0., 1.);

  t * t * (3. - 2. * t)
}
