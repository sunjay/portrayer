use rand::Rng;

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

/// A parallelogram defined by two not-necessarily-normalized vectors a and b. The area of the
/// parallelogram falls between (-a, -b) and (a, b). That is, all coordinates within the shape
/// fall in the range [-1, 1].
///
/// The normal of this parallelogram is a x b
#[derive(Debug, Clone, Default)]
pub struct Parallelogram {
    /// The first basis vector of the parallelogram
    pub a: Vec3,
    /// The second basis vector of the parallelogram
    pub b: Vec3,
}

impl Parallelogram {
    /// Returns true if this parallelogram has zero area
    pub fn is_empty(&self) -> bool {
        self.a == Vec3::zero() || self.b == Vec3::zero()
    }

    /// Returns the normal vector of the surface of this parallelogram (not guaranteed to be
    /// normalized)
    pub fn normal(&self) -> Vec3 {
        self.a.cross(self.b)
    }

    /// Sample a random point within the parallelogram
    pub fn sample_point<R: Rng>(&self, mut rng: R) -> Vec3 {
        let Parallelogram {a, b} = *self;

        // Compute two coordinates between -1 and 1
        let a_coord = 2.0 * rng.gen::<f64>() - 1.0;
        let b_coord = 2.0 * rng.gen::<f64>() - 1.0;

        a_coord * a + b_coord * b
    }
}

#[derive(Debug, Clone, Default)]
pub struct Light {
    /// The position of the center of the light
    pub position: Vec3,
    /// The color and intensity of the light
    pub color: Rgb,
    /// The attenuation factor of the light (intensity drop off with distance)
    pub falloff: Falloff,
    /// The area of the light. If zero, the light is a point light. If non-zero, this area will be
    /// used to sample random points on the light and soften shadows.
    pub area: Parallelogram,
}

impl Light {
    /// Return a random position within the area of the light
    pub fn sample_position<R: Rng>(&self, rng: R) -> Vec3 {
        self.position + self.area.sample_point(rng)
    }
}
