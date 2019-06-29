use crate::math::{Vec3, Vec3Ext, Mat4, Radians};
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
    pub fn new(cam: CameraSettings, (width, height): (f64, f64)) -> Self {
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
    pub fn ray_at(&self, (x, y): (f64, f64)) -> Ray {
        // NDC = Normalized Device Coordinates

        // This function goes through 4 coordinate systems:

        // Screen Space  ==>  Ray Tracing NDC  ==>  View Space  ==>  World Space
        // o ------> x        o ------> x           ^ y (-1, 1)           ^ y
        // |      (0, w)      |      (0, 1)         |                     |
        // |                  |               ----- o -----> x      ----- o -----> x
        // v                  v                   / |    (-1, 1)        / |
        //   y (0, h)           y (0, 1)        -z  |  Right-handed   +z  |    Right-handed
        //
        // w = width, h = height, 3rd axis comes out of the screen towards you

        // Ray tracing NDC is between 0 and 1 (inclusive)
        let pixel_ndc_y = y / self.height;
        // Map to -1 to 1 (view space) (& flip axis)
        let pixel_view_y = (1.0 - 2.0*pixel_ndc_y) * self.fov_factor;

        // Ray tracing NDC is between 0 and 1 (inclusive)
        let pixel_ndc_x = x / self.width;
        // Map to -1 to 1 (view space)
        // Let y remain fixed, but scale x by the aspect ratio to get the right dimensions
        // (changes the range of x to be bigger than y if the aspect ratio > 1.0
        // or smaller if aspect ratio < 1.0)
        let pixel_view_x = (2.0*pixel_ndc_x - 1.0) * self.aspect_ratio * self.fov_factor;

        // Image plane is 1.0 unit ahead of the camera/eye in camera/view space.
        // Using -1.0 because view space is right-handed.
        let pixel_view = Vec3::new(pixel_view_x, pixel_view_y, -1.0);
        // Transform to world coordinates from camera space
        let pixel_world = pixel_view.transformed_point(self.view_to_world);
        // The ray goes from the eye to the pixel_world coordinate
        let ray_dir = (pixel_world - self.eye).normalized();

        Ray::new(self.eye, ray_dir)
    }
}
