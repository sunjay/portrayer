use crate::math::{Vec3, Rgb};
use crate::scene::Scene;

#[derive(Debug)]
pub struct Material {}

impl Material {
    /// Compute the color of a ray intersection using the lighting model of this material, possibly
    /// casting further rays to simulate things like reflection/refraction/etc.
    pub fn hit_color(
        &self,
        scene: &Scene,
        background: Rgb,
        ray_dir: Vec3,
        hit_point: Vec3,
        normal: Vec3,
    ) -> Rgb {
        unimplemented!()
    }
}
