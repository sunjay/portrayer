use crate::math::{Vec3, Rgb};

/// The light "fall off" value, used for attenuation
///
/// attenuated_light = light_value / (c0 + c1*r + c2*r^2)
///     where r = distance to light from the hit point
///
/// This allows light to become darker as the distance to the light increases
#[derive(Debug)]
pub struct Falloff {
    c0: f64,
    c1: f64,
    c2: f64,
}

impl Falloff {
    /// Returns the attenuation value at the given distance from the light to the hit point
    pub fn at_distance(&self, light_dist: f64) -> f64 {
        self.c0 + self.c1*light_dist + self.c2*light_dist*light_dist
    }
}

#[derive(Debug)]
pub struct Light {
    pub position: Vec3,
    pub color: Rgb,
    pub falloff: Falloff,
}
