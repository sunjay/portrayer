use crate::math::{Vec3, Mat4};
use crate::render::{Target, TargetInfo};

#[derive(Debug)]
pub struct Camera {
    /// Represents the transformation from the view space to the world space
    view_to_world: Mat4,
    fovy: f64,
}

impl Camera {
    pub fn new(eye: Vec3, target: Vec3, up: Vec3, fovy: f64) -> Self {
        unimplemented!()
    }

    /// Returns the ray at the given pixel (x, y) position from the given target
    pub fn ray_at<T: Target>(&self, (x, y): (usize, usize), target: &T) {
        let TargetInfo {width, height, ..} = target.target_info();
        assert!(x < width && y < height, "bug: pixel position out of range");

        unimplemented!()
    }
}
