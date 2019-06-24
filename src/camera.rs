use crate::math::{Vec3, Mat4, Radians};
use crate::render::{Target, TargetInfo};
use crate::ray::Ray;

#[derive(Debug, Clone, Copy)]
pub struct CameraSettings {
    /// The position of the camera in world space
    pub eye: Vec3,
    /// The target position that the camera is looking at in world space
    pub center: Vec3,
    /// The "up" direction of this camera
    pub up: Vec3,
    /// The field-of-view angle along the y-axis of the camera
    pub fovy: Radians,
}

#[derive(Debug)]
pub struct Camera {
    /// Represents the transformation from the view space to the world space
    view_to_world: Mat4,
    /// The field-of-view angle along the y-axis of the camera
    fovy: f64,
    /// The ratio width/height for a given target
    aspect_ratio: f64,
}

impl Camera {
    pub fn new<T: Target>(settings: CameraSettings, target: &T) -> Self {
        let TargetInfo {width, height, ..} = target.target_info();
        unimplemented!()
    }

    /// Returns the ray at the given pixel (x, y) position
    pub fn ray_at(&self, (x, y): (usize, usize)) -> Ray {
        unimplemented!()
    }
}
