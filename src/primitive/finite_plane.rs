use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Vec3, Uv};
use crate::bounding_box::{BoundingBox, Bounds};

use super::plane::Plane;

/// L = length/width of the plane (the height is 0.0)
const L: f64 = 1.0;
const L2: f64 = L / 2.0;

/// A flat, finite plane with center (0, 0, 0), length = 1.0, width = 1.0, and height = 0.0
///
/// The plane's normal faces "up", i.e. {x: 0.0, y: 1.0, z: 0.0}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinitePlane;

impl Bounds for FinitePlane {
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

impl RayHit for FinitePlane {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        Plane {normal: Vec3::up(), point: Vec3::zero()}
            .ray_hit(ray, t_range)
            .and_then(|mut hit| if contains(hit.hit_point) {
                hit.tex_coord = Some(Uv {
                    u: hit.hit_point.x + L2,
                    v: hit.hit_point.z + L2,
                });
                Some(hit)
            } else {
                None
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::math::Mat4;

    #[test]
    fn rotated_plane_bounds() {
        let plane = FinitePlane;

        let bounds = plane.bounds();
        assert_eq!(bounds.min(), Vec3 {x: -0.5, y: 0.0, z: -0.5});
        assert_eq!(bounds.max(), Vec3 {x: 0.5, y: 0.0, z: 0.5});

        let trans = Mat4::rotation_x(90.0f64.to_radians());
        let rotated_bounds = trans * bounds;
        assert_eq!(rotated_bounds.min().map(|x| (x * 10.0).round() / 10.0), Vec3 {x: -0.5, y: -0.5, z: 0.0});
        assert_eq!(rotated_bounds.max().map(|x| (x * 10.0).round() / 10.0), Vec3 {x: 0.5, y: 0.5, z: 0.0});
    }
}
