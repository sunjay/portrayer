use vek::ops::Clamp;

use crate::math::Rgb;
use crate::scene::Scene;
use crate::camera::{CameraSettings, Camera};
use crate::texture::Texture;

#[derive(Debug)]
pub struct TargetInfo {
    /// Returns the width of this target in pixels
    pub width: usize,
    /// Returns the height of this target in pixels
    pub height: usize,
}

/// A target that can be rendered to.
///
/// It is assumed that the target has pixels that can be indexed from
/// x = 0..target_info.width and y = 0..target.height. The image space is assumed to
/// go left to right on the x axis and top to bottom on the y axis.
pub trait Target: Sized {
    /// Returns information about the given target used for rendering
    fn target_info(&self) -> TargetInfo;

    /// Returns the color of the given pixel
    ///
    /// Unsafe for performance reasons: this method is allowed to skip bounds checking
    unsafe fn get_pixel(&self, x: usize, y: usize) -> Rgb;

    /// Sets the color of the given pixel to the given value
    ///
    /// Unsafe for performance reasons: this method is allowed to skip bounds checking
    unsafe fn set_pixel(&mut self, x: usize, y: usize, color: Rgb);

    /// Draw the given scene to this target using the given camera settings and background texture
    fn draw<T: Texture>(
        &mut self,
        scene: &Scene,
        camera: CameraSettings,
        background: T,
    ) {
        let camera = Camera::new(camera, self);
        let TargetInfo {width, height, ..} = self.target_info();

        for y in 0..height {
            for x in 0..width {
                let ray = camera.ray_at((x, y));

                let background_color = background.at(x as f64 / width as f64, y as f64 / height as f64);
                let color = ray.color(scene, background_color, 0);

                // Gamma correction to ensure that image colors are closer to what we want them
                // to be. This gamma value is the same as Blender and is also in the source below:
                // Source: https://learnopengl.com/Advanced-Lighting/Gamma-Correction
                let gamma = 2.2;
                let color = color.map(|c| c.powf(1.0/gamma));

                // Clamp to 0.0 to 1.0 or else we will get invalid pixels in the output PNG
                let color: Rgb = Clamp::<f64>::clamp01(color);

                // Unsafe because we are guaranteeing that the (x, y) value is in the valid range
                unsafe { self.set_pixel(x, y, color); }
            }
        }
    }
}

impl<T: Target> Target for &mut T {
    fn target_info(&self) -> TargetInfo {
        (**self).target_info()
    }

    unsafe fn get_pixel(&self, x: usize, y: usize) -> Rgb {
        (**self).get_pixel(x, y)
    }

    unsafe fn set_pixel(&mut self, x: usize, y: usize, color: Rgb) {
        (**self).set_pixel(x, y, color)
    }
}

impl Target for image::RgbImage {
    fn target_info(&self) -> TargetInfo {
        TargetInfo {
            width: self.width() as usize,
            height: self.height() as usize,
        }
    }

    unsafe fn get_pixel(&self, x: usize, y: usize) -> Rgb {
        let [r, g, b] = self.get_pixel(x as u32, y as u32).data;
        Rgb {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }

    unsafe fn set_pixel(&mut self, x: usize, y: usize, color: Rgb) {
        self.put_pixel(x as u32, y as u32, image::Rgb {
            data: [
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
            ],
        });
    }
}
