use crate::math::{Vec3, Vec3Ext, Mat4, Radians};
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
    /// The position of the camera in world space
    eye: Vec3,
    /// Represents the transformation from the view space to the world space
    view_to_world: Mat4,
    /// Allows us to scale by the fov to dictate how much of the world is shown in the
    /// rendered image (similar to zoom).
    fov_factor: f64,
    /// The ratio width/height for a given target
    aspect_ratio: f64,
    // The width of the target
    width: f64,
    // The height of the target
    height: f64,
}

impl Camera {
    pub fn new<T: Target>(cam: CameraSettings, target: &T) -> Self {
        let TargetInfo {width, height, ..} = target.target_info();
        let width = width as f64;
        let height = height as f64;

        Self {
            eye: cam.eye,
            // Need to invert because look_at returns a world-to-view matrix by default
            view_to_world: Mat4::look_at_rh(cam.eye, cam.center, cam.up).inverted(),
            // This assumes that the camera is 1.0 unit away from the image plane
            fov_factor: (cam.fovy.get()/2.0).tan(),
            aspect_ratio: width / height,
            width,
            height,
        }
    }

    /// Returns the primary ray at the given pixel (x, y) position
    pub fn ray_at(&self, (x, y): (usize, usize)) -> Ray {
        // NDC = Normalized Device Coordinates

        // Ray tracing NDC is between 0 and 1 (inclusive)
        // +0.5 so in the middle of the pixel square
        let pixel_ndc_y = (y as f64 + 0.5) / self.height;
        // Map to -1 to 1 (screen space) (& flip axis)
        let pixel_screen_y = (1.0 - 2.0*pixel_ndc_y) * self.fov_factor;

        // Ray tracing NDC is between 0 and 1 (inclusive)
        // +0.5 so in the middle of the pixel square
        let pixel_ndc_x = (x as f64 + 0.5) / self.width;
        // Map to -1 to 1 (screen space)
        // Let y remain fixed, but scale x by the aspect ratio to get the right dimensions
        // (changes the range of x to be bigger than y if the aspect ratio > 1.0
        // or smaller if aspect ratio < 1.0)
        let pixel_screen_x = (2.0*pixel_ndc_x - 1.0) * self.aspect_ratio * self.fov_factor;

        // Image plane is 1.0 unit ahead of the camera/eye in camera/view space.
        // Using -1.0 because right-handed.
        let pixel_camera = Vec3::new(pixel_screen_x, pixel_screen_y, -1.0);
        // Transform to world coordinates from camera space
        let pixel_world = pixel_camera.transformed_point(self.view_to_world);
        // The ray goes from the eye to the pixel_world coordinate
        let ray_dir = (pixel_world - self.eye).normalized();

        Ray::new(self.eye, ray_dir)
    }
}
