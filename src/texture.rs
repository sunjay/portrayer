use crate::math::Rgb;

pub trait Texture {
    /// Sample the texture at the given point
    ///
    /// # Panics
    ///
    /// The texture may panic if x or y are not in a valid range.
    fn at(&self, x: usize, y: usize) -> Rgb;
}

/// Allows any arbitrary function to be used as a texture as long as it has the signature:
///     fn(x: usize, y: usize) -> Rgb
impl<T> Texture for T where T: Fn(usize, usize) -> Rgb {
    fn at(&self, x: usize, y: usize) -> Rgb {
        (*self)(x, y)
    }
}
