use crate::graphics::{ray::Ray, surface::{Surface, Sphere}, vector::{Point3, Vec3}, color::Color};

pub const ASPECT_RATIO: f32 = 16.0 / 9.0;
pub const HEIGHT: u32 = 400;
pub const WIDTH: u32 = ((HEIGHT as f32) * ASPECT_RATIO) as u32;

const CAMERA_CENTER: Point3 = Point3{x: 0., y: 0., z: 0.};

const FOCAL_LENGTH: f32 = 1.0;
const VIEWPORT_HEIGHT: f32 = 2.0;
const VIEWPORT_WIDTH: f32 = VIEWPORT_HEIGHT * ASPECT_RATIO;
const MAX_DISTANCE: f32 = 10.;
const STEP_SIZE: f32 = 0.1;

pub struct World {
    pub objects: Vec<Box<dyn Surface>>,    
}

impl World {
    pub fn ray_color(&self, mut ray: Ray) -> Color {
        while (ray.pos - CAMERA_CENTER).length() < MAX_DISTANCE {
            ray = ray.step(STEP_SIZE);
            for obj in &self.objects {
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

    pub fn get_image(&self, frame: &mut [u8]) {

        let viewport_u_vect = Vec3{x: VIEWPORT_WIDTH, y: 0., z: 0.};
        let viewport_v_vect = Vec3{x: 0., y: -VIEWPORT_HEIGHT, z: 0.};

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u_vect / (WIDTH as f32);
        let pixel_delta_v = viewport_v_vect / (HEIGHT as f32);

        // Calculate the location of the upper left pixel.
        let viewport_upper_left = 
            CAMERA_CENTER - Vec3{x: 0., y: 0., z: FOCAL_LENGTH} - viewport_u_vect/2. - viewport_v_vect/2.;
        
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        for (idx, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let i = idx % WIDTH as usize;
            let j = idx / WIDTH as usize;

            let pixel_center = pixel00_loc + (pixel_delta_u * (i as f32)) + (pixel_delta_v * (j as f32));
            let ray_direction = pixel_center - CAMERA_CENTER;
            let ray = Ray{pos: CAMERA_CENTER, vel: ray_direction};

            let col = self.ray_color(ray);

            pixel[0] = col.r; // R
            pixel[1] = col.g; // G
            pixel[2] = col.b; // B
            pixel[3] = col.a; // A
        }
    }
}