use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Vec3, Uv, Mat3};
use crate::bounding_box::{BoundingBox, Bounds};

use super::InfinitePlane;

/// L = length/width of the plane (the height is 0.0)
const L: f64 = 1.0;
const L2: f64 = L / 2.0;

/// A flat, finite plane with center (0, 0, 0), length = 1.0, width = 1.0, and height = 0.0
///
/// The plane's normal faces "up", i.e. {x: 0.0, y: 1.0, z: 0.0}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plane;

impl Bounds for Plane {
    fn bounds(&self) -> BoundingBox {
        let min = Vec3 {x: -L2, y: 0.0, z: -L2};
        let max = Vec3 {x: L2, y: 0.0, z: L2};
        BoundingBox::new(min, max)
    }
}

/// Returns true if the given point is within the boundary of the plane
///
/// Only need to check two axes because third axis is guaranteed to be zero
fn contains(Vec3 {x, y: _, z}: Vec3) -> bool {
    let radius = L2 + EPSILON;
    -radius <= x && x <= radius && -radius <= z && z <= radius
}

impl RayHit for Plane {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        InfinitePlane {normal: Vec3::up(), point: Vec3::zero()}
            .ray_hit(ray, t_range)
            .and_then(|mut hit| if contains(hit.hit_point) {
                hit.tex_coord = Some(Uv {
                    u: hit.hit_point.x + L2,
                    v: hit.hit_point.z + L2,
                });

                // Normal direction is already oriented correctly
                hit.normal_map_transform = Some(Mat3::identity());

                Some(hit)
            } else {
                None
            })
    }
}
