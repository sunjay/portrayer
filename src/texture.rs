use std::fmt;
use std::path::Path;

use vek::Clamp;

use crate::math::{Uv, Rgb};

pub trait TextureSource {
    /// Sample the texture at the given point.
    ///
    /// Both components of uv are between 0.0 and 1.0.
    fn at(&self, uv: Uv) -> Rgb;
}

/// Allows any arbitrary function to be used as a texture as long as it has the signature:
///     fn(uv: Uv) -> Rgb
impl<T> TextureSource for T where T: Fn(Uv) -> Rgb {
    fn at(&self, uv: Uv) -> Rgb {
        (*self)(uv)
    }
}

#[derive(Debug)]
pub struct ImageTexture {
    buffer: image::RgbImage,
}

impl From<image::RgbImage> for ImageTexture {
    fn from(buffer: image::RgbImage) -> Self {
        Self {buffer}
    }
}

impl ImageTexture {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, image::ImageError> {
        let img = image::open(path)?.to_rgb();
        Ok(Self::from(img))
    }
}

impl TextureSource for ImageTexture {
    fn at(&self, uv: Uv) -> Rgb {
        // Need to clamp to 0.0 to 1.0 to account for floating point error and ensure we never
        // accidentally index out of bounds
        let uv = Clamp::<f64>::clamp01(uv);
        // Need to subtract 1 because the final index is width - 1 and height - 1
        let x = (uv.u * (self.buffer.width() - 1) as f64) as u32;
        let y = (uv.v * (self.buffer.height() - 1) as f64) as u32;
        let [r, g, b] = self.buffer.get_pixel(x, y).data;

        Rgb {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }
}

pub enum Texture {
    /// A texture created from a function
    FnTex(Box<Fn(Uv) -> Rgb + Send + Sync>),
    Image(ImageTexture),
}

impl<T> From<T> for Texture where T: Fn(Uv) -> Rgb + Send + Sync + 'static {
    fn from(f: T) -> Self {
        Texture::FnTex(Box::new(f))
    }
}

impl From<ImageTexture> for Texture {
    fn from(img: ImageTexture) -> Self {
        Texture::Image(img)
    }
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Texture::*;
        match self {
            FnTex(_) => f.debug_tuple("FnTex").field(&format_args!("<function>")).finish(),
            Image(image) => image.fmt(f),
        }
    }
}

impl TextureSource for Texture {
    fn at(&self, uv: Uv) -> Rgb {
        use Texture::*;
        match self {
            FnTex(f) => f.at(uv),
            Image(img) => img.at(uv),
        }
    }
}
