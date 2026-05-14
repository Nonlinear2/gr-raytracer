use crate::graphics::{ray::Ray, vector::{Point3, Vec3, Color}, world::World};

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
            max_distance: 7.,
            ray_step_size: 0.01,
        }
    }

    pub fn aspect_ratio(&self) -> f32{
        self.viewport_width / self.viewport_height
    }

    pub fn ray_color(&self, mut ray: Ray, depth: u32, world: &World) -> Color {
        if depth <= 0 {
            return Color {x: 0., y: 0., z: 0.};
        }

        while (ray.pos - self.center).length() < self.max_distance {
            ray = ray.step(self.ray_step_size);
            for obj in &world.objects {
                if let Some(hit) = obj.hit(&ray) {
                    let bounced = self.ray_color(
                        Ray {
                            pos: hit.point + 0.001 * hit.normal,
                            vel: (hit.normal + Vec3::random_on_sphere()/2.).normalize(),
                        },
                        depth - 1,
                        world,
                    );

                    return hit.material.emission
                        + Color {
                            x: hit.material.color.x * bounced.x / 255.0,
                            y: hit.material.color.y * bounced.y / 255.0,
                            z: hit.material.color.z * bounced.z / 255.0,
                        };
                }
            }
        }

        let a = 0.5 * (ray.vel.normalize().y + 1.0);
        return (1.-a)*(Color {x: 255.0, y: 255.0, z: 255.0}) + a*(Color {x: 127.0, y: 190.0, z: 255.0});
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

            let col = self.ray_color(ray, 3, &world);

            pixel[0] = col.x as u8; // R
            pixel[1] = col.y as u8; // G
            pixel[2] = col.z as u8; // B
            pixel[3] = 0xff; // A
        }
    }
}