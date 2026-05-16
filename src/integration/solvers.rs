pub fn positive_root(a: f32, b: f32, c: f32) -> f32 {
    let delta = b*b - 4.*a*c;
    assert!(delta > 0.);

    (-b + delta.sqrt()) / (2.0 * a)
}