use std::io;
use std::env;
use std::path::{Path, PathBuf};

use vek::ops::Clamp;
use rayon::prelude::*;
use image::Pixel;
use rand::{Rng, thread_rng};

use crate::math::{GAMMA, Uv, Rgb};
use crate::scene::{Scene, HierScene};
#[cfg(any(feature = "kdtree", feature = "flat_scene"))]
use crate::flat_scene::FlatScene;
#[cfg(feature = "kdtree")]
use crate::kdtree::KDTreeScene;
use crate::ray::RayCast;
use crate::camera::{CameraSettings, Camera};
use crate::texture::TextureSource;
use crate::reporter::Reporter;

/// Ray traces a single pixel through the scene
fn render_single_pixel<R: RayCast + Send + Sync, T: TextureSource>(
    (x, y): (usize, usize),
    scene: &Scene<R>,
    camera: &Camera,
    width: f64,
    height: f64,
    samples: usize,
    background: &T,
) -> Rgb {
    let background_color = background.at(Uv {
        u: x as f64 / width,
        v: y as f64 / height,
    });

    let total_color: Rgb = (0..samples).into_par_iter().panic_fuse().map(|_| {
        // Choose a random point in the pixel square
        let mut rng = thread_rng();
        let (x, y) = (x as f64 + rng.gen::<f64>(), y as f64 + rng.gen::<f64>());
        let ray = camera.ray_at((x, y));

        ray.color(scene, background_color, 0)
    }).reduce(|| Rgb::black(), |x, y| x + y);

    let color = total_color / samples as f64;

    let color = color.map(|c| c.powf(1.0/GAMMA));

    // Clamp to 0.0 to 1.0 or else we will get invalid pixels in the output PNG
    Clamp::<f64>::clamp01(color)
}

/// Represents a 2D slice of an image
///
/// x and y are 0-indexed. x is left-to-right across and y is top-to-bottom down the image.
pub struct ImageSliceMut<'a> {
    image: &'a mut Image,
    /// The (x, y) coordinate of the top left of the slice
    ///
    /// This is guaranteed to be inside the image, but not guaranteed to be less than bottom_right
    top_left: (usize, usize),
    /// The (x, y) of the bottom right of the slice
    ///
    /// This is guaranteed to be inside the image, but not guaranteed to be greater than top_left
    bottom_right: (usize, usize),
}

impl<'a> From<&'a mut Image> for ImageSliceMut<'a> {
    fn from(image: &'a mut Image) -> Self {
        let width = image.width();
        let height = image.height();
        Self::new(image, (0, 0), (width - 1, height - 1))
    }
}

impl<'a> ImageSliceMut<'a> {
    /// Creates a new image slice from the given image and panics if either of the given (x, y)
    /// positions are out of bounds
    pub fn new(image: &'a mut Image, top_left: (usize, usize), bottom_right: (usize, usize)) -> Self {
        let width = image.width();
        let height = image.height();
        let (x1, y1) = top_left;
        let (x2, y2) = bottom_right;
        if x1 >= width || y1 >= height || x2 >= width || y2 >= height {
            panic!("The positions {{x: {}, y: {}}} and/or {{x: {}, y: {}}} are not within an image with width = {} and height = {}",
                x1, y1, x2, y2, width, height);
        }

        Self {image, top_left, bottom_right}
    }

    /// Render the given scene onto the entirety of this image
    pub fn render<R: Reporter + Send + Sync, T: TextureSource + Send + Sync>(
        &mut self,
        scene: &HierScene,
        camera: CameraSettings,
        background: T,
    ) {
        let width = self.image.width() as f64;
        let height = self.image.height() as f64;
        let camera = Camera::new(camera, (width, height));

        let reporter = R::new((self.image.width() * self.image.height()) as u64);

        // Attempt to get the number of samples from an environment variable, and ignore the value
        // otherwise
        let samples = env::var("SAMPLES").ok()
            // Must be a valid number
            .and_then(|val| val.parse::<usize>().ok())
            // Must be positive (greater than zero)
            .and_then(|val| if val > 0 { Some(val) } else { None })
            // Default value if not all conditions are met
            .unwrap_or(100);

        // Only render the sliced pixels
        let (x1, y1) = self.top_left;
        let (x2, y2) = self.bottom_right;
        let x_range = x1..=x2;
        let y_range = y1..=y2;

        #[cfg(feature = "flat_scene")]
        let scene = &FlatScene::from(scene);
        #[cfg(feature = "kdtree")]
        let flat_scene = FlatScene::from(scene);
        #[cfg(feature = "kdtree")]
        let scene = &KDTreeScene::from(flat_scene);
        self.image.buffer.par_chunks_mut(3)
            .map(image::Rgb::from_slice_mut)
            .enumerate()
            .panic_fuse()
            .for_each(|(i, pixel)| {
                let x = i % width as usize;
                let y = i / width as usize;

                // Skip any pixels not in the range
                if !x_range.contains(&x) || !y_range.contains(&y) {
                    return;
                }

                let color = render_single_pixel((x, y), scene, &camera, width, height, samples, &background);

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

pub struct Image {
    path: PathBuf,
    buffer: image::RgbImage,
}

impl Image {
    /// Attempts to open the given image path as an RGB image.
    ///
    /// If the file does not exist or if the dimensions are different, a new file will be created
    /// with the given path and dimensions. This allows you to preserve the image if only drawing
    /// on a limited slice of it.
    pub fn new<P: AsRef<Path>>(path: P, width: usize, height: usize) -> image::ImageResult<Self> {
        let path = path.as_ref();
        let buffer = match image::open(path) {
            Ok(image) => {
                let buffer = image.to_rgb();
                if buffer.width() == width as u32 && buffer.height() == height as u32 {
                    buffer
                } else {
                    // Wrong dimensions: Create a new buffer
                    image::RgbImage::new(width as u32, height as u32)
                }
            },
            Err(image::ImageError::IoError(ref err)) if err.kind() == io::ErrorKind::NotFound => {
                // Image does not exist: Create a new buffer
                image::RgbImage::new(width as u32, height as u32)
            },
            Err(err) => return Err(err),
        };

        Ok(Self {
            path: path.to_path_buf(),
            buffer,
        })
    }

    /// Returns the width of this image
    pub fn width(&self) -> usize {
        self.buffer.width() as usize
    }

    /// Returns the height of this image
    pub fn height(&self) -> usize {
        self.buffer.height() as usize
    }

    /// Attempts to save/update the image
    pub fn save(&self) -> io::Result<()> {
        self.save_as(&self.path)
    }

    /// Attempts to save the image at the given path
    pub fn save_as<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        self.buffer.save(path)
    }

    /// Returns a mutable slice to the area of the image between the given (x, y) pairs
    pub fn slice_mut(&mut self, top_left: (usize, usize), bottom_right: (usize, usize)) -> ImageSliceMut {
        ImageSliceMut::new(self, top_left, bottom_right)
    }

    /// Render the given scene onto the entirety of this image
    pub fn render<R: Reporter + Send + Sync, T: TextureSource + Send + Sync>(
        &mut self,
        scene: &HierScene,
        camera: CameraSettings,
        background: T,
    ) {
        ImageSliceMut::from(self).render::<R, _>(scene, camera, background)
    }
}
