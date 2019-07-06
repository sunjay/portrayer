use crate::math::{Vec3, Rgb};

/// The light "fall off" value, used for attenuation
///
/// attenuated_light = light_value / (c0 + c1*r + c2*r^2)
///     where r = distance to light from the hit point
///
/// This allows light to become darker as the distance to the light increases
#[derive(Debug, Clone)]
pub struct Falloff {
    pub c0: f64,
    pub c1: f64,
    pub c2: f64,
}

impl Default for Falloff {
    fn default() -> Self {
        Self {
            // Results in attenuation = 1.0 (no effect)
            c0: 1.0,
            c1: 0.0,
            c2: 0.0,
        }
    }
}

impl Falloff {
    /// Returns the attenuation value at the given distance from the light to the hit point
    pub fn at_distance(&self, light_dist: f64) -> f64 {
        self.c0 + self.c1*light_dist + self.c2*light_dist*light_dist
    }
}

#[derive(Debug, Clone)]
pub struct Light {
    pub position: Vec3,
    pub color: Rgb,
    pub falloff: Falloff,
}
