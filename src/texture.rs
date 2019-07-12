use std::fmt;
use std::path::Path;

use vek::Clamp;

use crate::math::{Uv, Rgb, Vec3, Mat3};

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

/// A type that encapsulates all supported texture types
pub enum Texture {
    /// A texture created from a function
    FnTex(Box<Fn(Uv) -> Rgb + Send + Sync>),
    /// A texture created from an image
    Image(ImageTexture),
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

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        use Texture::*;
        match (self, other) {
            //TODO: Not actually used in tests so I will implement this when it is needed
            (FnTex(_), FnTex(_)) => unimplemented!(),
            (Image(img), Image(img2)) => img.eq(&img2),
            _ => false,
        }
    }
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

impl TextureSource for Texture {
    fn at(&self, uv: Uv) -> Rgb {
        use Texture::*;
        match self {
            FnTex(f) => f.at(uv),
            Image(img) => img.at(uv),
        }
    }
}

/// A texture where each point is sampled from an image
pub struct ImageTexture {
    buffer: image::RgbImage,
}

impl fmt::Debug for ImageTexture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("ImageTexture").field(&format_args!("..")).finish()
    }
}

impl PartialEq for ImageTexture {
    fn eq(&self, other: &Self) -> bool {
        self.buffer.eq(&*other.buffer)
    }
}

impl From<image::RgbImage> for ImageTexture {
    fn from(buffer: image::RgbImage) -> Self {
        Self {buffer}
    }
}

impl ImageTexture {
    /// Creates an image texture from the image file at the given path
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

/// Interprets normals loaded from a texture
///
/// The normals in the texture are assumed to be in a left-handed coordinate system where a normal
/// that is perpendicular to the surface points along the -Z axis.
#[derive(Debug, PartialEq)]
pub struct NormalMap {
    texture: ImageTexture,
}

impl NormalMap {
    /// Creates a normal map that samples from a texture made from the image file at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, image::ImageError> {
        Ok(Self {
            texture: ImageTexture::open(path)?,
        })
    }

    /// Loads a normal from the texture and transforms it so that it is in the same right-handed
    /// coordinate system as the rest of the ray tracer. A normal perpendicular to the surface will
    /// point along the +Y axis.
    ///
    /// The returned normal is normalized if the normals in the texture are normalized.
    pub fn normal_at(&self, uv: Uv) -> Vec3 {
        // The color loaded from the texture map needs to be converted to a vector
        // using the following mapping:
        //
        // X: -1 to +1 :  Red:     0 to 255 (0.0 to 1.0)
        // Y: -1 to +1 :  Green:   0 to 255 (0.0 to 1.0)
        // Z:  0 to -1 :  Blue:  128 to 255 (0.5 to 1.0)
        //
        // Source: https://en.wikipedia.org/wiki/Normal_mapping#Interpreting_Tangent_Space_Maps
        let tex_norm = self.texture.at(uv);
        let norm = Vec3 {
            x: 2.0 * tex_norm.r - 1.0,
            y: 2.0 * tex_norm.g - 1.0,
            z: -(2.0 * tex_norm.b - 1.0),
        };

        // Normals in a normal map are oriented so that a normal perpendicular to the
        // surface will point along the -Z axis of a left-handed coordinate system.
        //
        // This matrix takes the normal map normal (nx,ny,nz) and turns it into (nx,-nz,-ny).
        // This makes a normal perpendicular to the surface point in the +Y direction
        // of a right-handed coordinate system.
        let normal_to_rh = Mat3::new(
            1.0, 0.0, 0.0, // * (nx, ny, nz) = nx
            0.0, 0.0, -1.0, // * (nx, ny, nz) = -nz
            0.0, -1.0, 0.0, // * (nx, ny, nz) = -ny
        );

        normal_to_rh * norm
    }
}
