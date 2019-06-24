use crate::math::Rgb;
use crate::ray::RayIntersection;

#[derive(Debug)]
pub struct Material {}

impl Material {
    /// Compute the color of a ray intersection, possibly casting further rays to simulate things
    /// like reflection/refraction/etc.
    pub fn hit_color(&self, ambient: Rgb) -> Rgb {
        unimplemented!()
    }
}
