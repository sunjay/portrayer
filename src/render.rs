use vek::ops::Clamp;
use rayon::prelude::*;
use image::Pixel;

use crate::math::{Uv, Rgb};
use crate::scene::{Scene, HierScene};
use crate::flat_scene::FlatScene;
use crate::kdtree::KDTreeScene;
use crate::ray::RayCast;
use crate::camera::{CameraSettings, Camera};
use crate::texture::TextureSource;
use crate::reporter::Reporter;

/// An extension trait for adding a render method to supported render targets
///
/// It is assumed that the render target has pixels that can be indexed from
/// x = 0..width and y = 0..height. The image space is assumed to
/// go left to right on the x axis and top to bottom on the y axis.
pub trait Render {
    /// Draw the given scene to this target using the given camera settings and background texture
    fn render<R: Reporter + Send + Sync, T: TextureSource + Send + Sync>(
        &mut self,
        scene: &HierScene,
        camera: CameraSettings,
        background: T,
    );
}

/// Ray traces a single pixel through the scene
fn render_single_pixel<R: RayCast, T: TextureSource>(
    (x, y): (f64, f64),
    scene: &Scene<R>,
    camera: &Camera,
    width: f64,
    height: f64,
    background: &T,
) -> Rgb {
    let ray = camera.ray_at((x, y));

    let background_color = background.at(Uv {
        u: x as f64 / width,
        v: y as f64 / height,
    });
    let color = ray.color(scene, background_color, 0);

    // Gamma correction to ensure that image colors are closer to what we want them
    // to be. This gamma value is the same as Blender and is also in the source below:
    // Source: https://learnopengl.com/Advanced-Lighting/Gamma-Correction
    let gamma = 2.2;
    let color = color.map(|c| c.powf(1.0/gamma));

    // Clamp to 0.0 to 1.0 or else we will get invalid pixels in the output PNG
    Clamp::<f64>::clamp01(color)
}

impl Render for image::RgbImage {
    fn render<R: Reporter + Send + Sync, T: TextureSource + Send + Sync>(
        &mut self,
        scene: &HierScene,
        camera: CameraSettings,
        background: T,
    ) {
        let width = self.width() as f64;
        let height = self.height() as f64;
        let camera = Camera::new(camera, (width, height));

        let reporter = R::new((self.width() * self.height()) as u64);

        // let scene = &FlatScene::from(scene);
        let flat_scene = FlatScene::from(scene);
        let scene = &KDTreeScene::from(flat_scene);
        self.par_chunks_mut(3)
            .map(image::Rgb::from_slice_mut)
            .enumerate()
            .for_each(|(i, pixel)| {
                let x = i % width as usize;
                let y = i / width as usize;

                // +0.5 so in the middle of the pixel square
                let (x, y) = (x as f64 + 0.5, y as f64 + 0.5);
                let color = render_single_pixel((x, y), scene, &camera, width, height, &background);

                // Convert into the type supported by the image library and write the pixel
                *pixel = image::Rgb([
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8,
                ]);

                reporter.report_finished_pixels(1);
            });
    }
}
