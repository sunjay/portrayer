use crate::math::Rgb;
use crate::scene::SceneNode;
use crate::camera::{CameraSettings, Camera};
use crate::light::Light;

#[derive(Debug)]
pub struct TargetInfo {
    /// Returns the width of this target in pixels
    pub width: usize,
    /// Returns the height of this target in pixels
    pub height: usize,
    /// The gamma value to use for gamma correction (e.g. 2.2)
    pub gamma: f64,
}

/// A target that can be rendered to.
///
/// It is assumed that the target has pixels that can be indexed from
/// x = 0..target_info.width and y = 0..target.height. The image space is assumed to
/// go left to right on the x axis and top to bottom on the y axis.
pub trait Target {
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

/// Represents the complete parameters needed to render an image
#[derive(Debug)]
pub struct RenderSettings<'a> {
    pub scene: &'a SceneNode,
    pub camera: CameraSettings,
    pub lights: &'a [Light],
    pub ambient: Rgb,
}

impl<'a> RenderSettings<'a> {
    /// Render the configured scene to the given target using the configured settings
    pub fn render<T: Target>(&self, target: &mut T) {
        let camera = Camera::new(self.camera, target);
        unimplemented!()
    }
}
