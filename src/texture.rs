use crate::math::Rgb;

pub trait Texture {
    /// Sample the texture at the given point.
    ///
    /// Both x and y are between 0.0 and 1.0.
    fn at(&self, x: f64, y: f64) -> Rgb;
}

/// Allows any arbitrary function to be used as a texture as long as it has the signature:
///     fn(x: f64, y: f64) -> Rgb
impl<T> Texture for T where T: Fn(f64, f64) -> Rgb {
    fn at(&self, x: f64, y: f64) -> Rgb {
        (*self)(x, y)
    }
}
