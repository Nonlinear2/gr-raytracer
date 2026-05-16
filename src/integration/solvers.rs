pub fn positive_root(a: f32, b: f32, c: f32) -> f32 {
    let mut delta = b*b - 4.0*a*c;

    // tolerate tiny negatives due to rounding
    if delta < 0.0 && delta > -1e-6 {
        delta = 0.0;
    }

    if delta < 0.0 {
        eprintln!(" negative discriminant delta={}", delta);
        // fallback: return the best real-valued estimate (use -b/(2a))
        return -b / (2.0 * a);
    }

    (-b + delta.sqrt()) / (2.0 * a)
}