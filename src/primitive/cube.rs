use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Vec3};

use super::plane::Plane;

/// An axis-aligned unit cube with center (0, 0, 0) and width/height/depth 1.0
///
/// It is expected that this cube will be used via affine transformations on the node that
/// contains it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cube;

/// Returns true if the given point is anywhere within the *volume* of the cube
fn contains(Vec3 {x, y, z}: Vec3) -> bool {
    // Need to add epsilon when doing these checks to account for floating point error. Without
    // this we get lots of "unfilled" spots ("shadow acne") all over the cube and its shadow.
    let radius = 0.5 + EPSILON;
    -radius <= x && x <= radius && -radius <= y && y <= radius && -radius <= z && z <= radius
}

impl RayHit for Cube {
    fn ray_hit(&self, ray: &Ray, init_t_range: &Range<f64>) -> Option<RayIntersection> {
        // Define the six faces of a cube
        static FACES: [Plane; 6] = [
            // Right
            Plane {point: Vec3 {x: 0.5, y: 0.0, z: 0.0}, normal: Vec3 {x: 1.0, y: 0.0, z: 0.0}},
            // Left
            Plane {point: Vec3 {x: -0.5, y: 0.0, z: 0.0}, normal: Vec3 {x: -1.0, y: 0.0, z: 0.0}},
            // Top
            Plane {point: Vec3 {x: 0.0, y: 0.5, z: 0.0}, normal: Vec3 {x: 0.0, y: 1.0, z: 0.0}},
            // Bottom
            Plane {point: Vec3 {x: 0.0, y: -0.5, z: 0.0}, normal: Vec3 {x: 0.0, y: -1.0, z: 0.0}},
            // Near
            Plane {point: Vec3 {x: 0.0, y: 0.0, z: 0.5}, normal: Vec3 {x: 0.0, y: 0.0, z: 1.0}},
            // Far
            Plane {point: Vec3 {x: 0.0, y: 0.0, z: -0.5}, normal: Vec3 {x: 0.0, y: 0.0, z: -1.0}},
        ];

        //TODO: Experiment with parallelism via rayon (might not be worth it for 6 checks)

        // Find the nearest intersection
        let mut t_range = init_t_range.clone();
        FACES.iter().fold(None, |hit, plane| {
            match plane.ray_hit(ray, &t_range) {
                // Need to check if the cube actually contains the hit point since each
                // plane is infinite
                Some(p_hit) => if contains(p_hit.hit_point) {
                    t_range.end = p_hit.ray_parameter;
                    Some(p_hit)
                } else { hit },
                None => hit,
            }
        })
    }
}
