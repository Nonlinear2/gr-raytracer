use crate::graphics::{ray::{Color, Ray}, vector::{Point3, Vec3}, world::World};

pub struct Camera {
    pub center: Point3,
    pub focal_length: f32,
    pub viewport_height: f32,
    pub viewport_width: f32,
    pub max_distance: f32,
    pub ray_step_size: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            center: Point3{x: 0., y: 0., z: 0.},
            focal_length: 1.0,
            viewport_height: 2.0,
            viewport_width: 2.0 * 16.0 / 9.0,
            max_distance: 15.,
            ray_step_size: 0.1,
        }
    }

    pub fn aspect_ratio(&self) -> f32{
        self.viewport_width / self.viewport_height
    }

    pub fn ray_color(&self, mut ray: Ray, world: &World) -> Color {
        while (ray.pos - self.center).length() < self.max_distance {
            ray = ray.step(self.ray_step_size);
            for obj in &world.objects {
                if let Some(hit) = obj.hit(&ray) {
                    return Color {
                        r: (255. * hit.normal.dot(&ray.vel).abs()) as u8,
                        g: 0,
                        b: 0,
                        a: 255
                    }
                }
            }
        }
        Color { r: ((ray.vel.normalize().y + 1.)*127.) as u8, g: 255, b: 255, a: 255 }
    }

    pub fn render(&self, frame: &mut [u8], width: u32, height: u32, world: &World) {
        assert!(width == (self.aspect_ratio() * (height as f32)) as u32);

        let viewport_u_vect = Vec3{x: self.viewport_width, y: 0., z: 0.};
        let viewport_v_vect = Vec3{x: 0., y: -self.viewport_height, z: 0.};

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u_vect / (width as f32);
        let pixel_delta_v = viewport_v_vect / (height as f32);

        // Calculate the location of the upper left pixel.
        let viewport_upper_left = 
            self.center - Vec3{x: 0., y: 0., z: self.focal_length} - viewport_u_vect/2. - viewport_v_vect/2.;
        
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        for (idx, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let i = idx % width as usize;
            let j = idx / width as usize;

            let pixel_center = pixel00_loc + (pixel_delta_u * (i as f32)) + (pixel_delta_v * (j as f32));
            let ray_direction = pixel_center - self.center;
            let ray = Ray{pos: self.center, vel: ray_direction};

            let col = self.ray_color(ray, &world);

            pixel[0] = col.r; // R
            pixel[1] = col.g; // G
            pixel[2] = col.b; // B
            pixel[3] = col.a; // A
        }
    }
}