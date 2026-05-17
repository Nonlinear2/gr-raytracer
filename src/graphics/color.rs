use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0 };
    pub const WHITE: Self = Self { r: 255.0, g: 255.0, b: 255.0 };

    pub fn new(r: f32, g: f32, b: f32) -> Self { Self { r, g, b } }

    pub fn from_vec3(v: Vec3) -> Self { Self { r: v.x, g: v.y, b: v.z } }
    pub fn as_vec3(&self) -> Vec3 { Vec3::new(self.r, self.g, self.b) }
}

impl std::ops::Add for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Color { Color::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b) }
}

impl std::ops::AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) { self.r += rhs.r; self.g += rhs.g; self.b += rhs.b; }
}

impl std::ops::Sub for Color {
    type Output = Color;
    fn sub(self, rhs: Color) -> Color { Color::new(self.r - rhs.r, self.g - rhs.g, self.b - rhs.b) }
}

impl std::ops::Mul<f32> for Color {
    type Output = Color;
    fn mul(self, rhs: f32) -> Color { Color::new(self.r * rhs, self.g * rhs, self.b * rhs) }
}

impl std::ops::Mul<Color> for f32 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Color { Color::new(rhs.r * self, rhs.g * self, rhs.b * self) }
}

impl std::ops::Div<f32> for Color {
    type Output = Color;
    fn div(self, rhs: f32) -> Color { Color::new(self.r / rhs, self.g / rhs, self.b / rhs) }
}

impl std::ops::DivAssign<f32> for Color {
    fn div_assign(&mut self, rhs: f32) { self.r /= rhs; self.g /= rhs; self.b /= rhs; }
}

impl From<Vec3> for Color { fn from(v: Vec3) -> Self { Color::from_vec3(v) } }
impl From<Color> for Vec3 { fn from(c: Color) -> Vec3 { Vec3::new(c.r, c.g, c.b) } }