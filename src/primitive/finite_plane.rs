use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Vec3};

use super::Plane;

/// L = length/width of the plane (the height is 0.0)
const L: f64 = 1.0;
const L2: f64 = L / 2.0;

/// A flat, finite plane with length = 1.0, width = 1.0, and height = 0.0
///
/// The plane's normal faces "up", i.e. {x: 0.0, y: 1.0, z: 0.0}
#[derive(Debug)]
pub struct FinitePlane;

/// Returns true if the given point is within the boundary of the plane
///
/// Only need to check two axes because third axis is guaranteed to be zero
fn contains(Vec3 {x, y: _, z}: Vec3) -> bool {
    let radius = L2 + EPSILON;
    -radius <= x && x <= radius && -radius <= z && z <= radius
}

impl RayHit for FinitePlane {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        Plane {normal: Vec3::up(), point: Vec3::zero()}
            .ray_hit(ray, t_range)
            .and_then(|hit| if contains(hit.hit_point) {
                Some(hit)
            } else {
                None
            })
    }
}
