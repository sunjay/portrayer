use std::ops::Range;

use crate::math::Vec3;

#[derive(Debug)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

/// Represents the result of a ray intersection and stores information about it
#[derive(Debug)]
pub struct RayIntersection {
    /// The smallest positive value of t for which the given ray intersects the target. Note that
    /// the smaller the t value, the closer the intersection is to the origin of the ray.
    ray_parameter: f64,

    /// The point of intersection
    hit_point: Vec3,

    /// The normal at the point of intersection.
    /// IMPORTANT: This is NOT guaranteed to be a unit vector for the sake of efficiency and
    /// floating point correctness. (Normalizing too many times accrues too much floating point
    /// error.) Make sure you normalize when it matters.
    normal: Vec3,
}

pub trait RayHit {
    /// Returns a value if the given ray has hit this object and the parameter is in the given range
    fn ray_hit(&self, ray: &Ray, range: &Range<f64>) -> Option<RayIntersection>;
}
