use crate::math::Rgb;
use crate::scene::SceneNode;

pub struct TargetInfo {
    /// Returns the width of this target in pixels
    pub width: usize,
    /// Returns the height of this target in pixels
    pub height: usize,
}

/// A target that can be rendered to
pub trait Target {
    /// Returns information about the given target used for rendering
    fn target_info(&self) -> TargetInfo;
    /// Returns the color of the given pixel
    fn get_pixel(&self, x: usize, y: usize) -> Rgb;
    /// Sets the color of the given pixel to the given value
    fn set_pixel(&mut self, x: usize, y: usize, color: Rgb);
}

impl<T: Target> Target for &mut T {
    fn target_info(&self) -> TargetInfo {
        (**self).target_info()
    }

    fn get_pixel(&self, x: usize, y: usize) -> Rgb {
        (**self).get_pixel(x, y)
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: Rgb) {
        (**self).set_pixel(x, y, color)
    }
}

/// Represents the complete parameters needed to render an image
pub struct RenderSettings<'a, T: Target> {
    pub scene: &'a SceneNode,
    pub target: T,
}

impl<'a, T: Target> RenderSettings<'a, T> {
    pub fn render(self) {
        unimplemented!()
    }
}
