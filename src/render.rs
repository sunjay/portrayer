use vek::ops::Clamp;

use crate::math::Rgb;
use crate::scene::Scene;
use crate::camera::{CameraSettings, Camera};
use crate::texture::Texture;

/// An extension trait for adding a render method to supported render targets
///
/// It is assumed that the render target has pixels that can be indexed from
/// x = 0..width and y = 0..height. The image space is assumed to
/// go left to right on the x axis and top to bottom on the y axis.
pub trait Render {
    /// Draw the given scene to this target using the given camera settings and background texture
    fn render<T: Texture>(&mut self, scene: &Scene, camera: CameraSettings, background: T);
}

impl Render for image::RgbImage {
    fn render<T: Texture>(&mut self, scene: &Scene, camera: CameraSettings, background: T) {
        let width = self.width() as f64;
        let height = self.height() as f64;
        let camera = Camera::new(camera, (width, height));

        for (x, y, pixel) in self.enumerate_pixels_mut() {
            // +0.5 so in the middle of the pixel square
            let ray = camera.ray_at((x as f64 + 0.5, y as f64 + 0.5));

            let background_color = background.at(x as f64 / width, y as f64 / height);
            let color = ray.color(scene, background_color, 0);

            // Gamma correction to ensure that image colors are closer to what we want them
            // to be. This gamma value is the same as Blender and is also in the source below:
            // Source: https://learnopengl.com/Advanced-Lighting/Gamma-Correction
            let gamma = 2.2;
            let color = color.map(|c| c.powf(1.0/gamma));

            // Clamp to 0.0 to 1.0 or else we will get invalid pixels in the output PNG
            let color: Rgb = Clamp::<f64>::clamp01(color);

            // Convert into the type supported by the image library and write the pixel
            *pixel = image::Rgb([
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
            ]);
        }
    }
}
