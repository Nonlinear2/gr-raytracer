pub fn positive_root(a: f32, b: f32, c: f32) -> f32 {
    let eps = 1e-8_f32;

    if a.abs() < eps {
        if b.abs() < eps {
            eprintln!("a and b close to zero");
            return 0.0;
        }
        return (-c / b).clamp(-1e6, 1e6);
    }

    let delta = b * b - 4.0 * a * c;

    if delta < 0.0 {
        eprintln!(" negative discriminant delta={}", delta);
        return (-b / (2.0 * a)).clamp(-1e6, 1e6);
    }

    return ((-b + delta.sqrt()) / (2.0*a)).clamp(0., 1e6);
}