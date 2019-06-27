use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Vec3};

/// A flat, infinite plane
#[derive(Debug)]
pub struct Plane {
    /// The normal of the plane (MUST be a unit vector)
    ///
    /// Assumption: In order to be visible, the normal must point *towards* the view
    pub normal: Vec3,
    /// Any point on the plane
    pub point: Vec3,
}

impl RayHit for Plane {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // Formula provided in the Graphics Codex, Ray Casting Chapter, Section 3
        // http://graphicscodex.com
        // Can be derived by substituting ray into implicit plane equation.

        let ray_dir = ray.direction();

        // Four cases where intersection can fail
        // 1. n.dot(d) == 0 (plane is perpendicular to the normal / parallel with the plane)
        // 2. (origin - point).dot(n) == 0 (ray is entirely in the plane)
        // 3. t < 0 (intersected in the past)
        // 4. n.dot(d) > 0 (intersection with the back of the plane face)
        //
        // 1 and 4 are checked for directly in the next step. 2 and 3 are caught by checking if t
        // is in t_range since t_range is typically (EPSILON, some positive value).

        let dot_dir_normal = ray_dir.dot(self.normal);
        if dot_dir_normal >= -EPSILON { // >= 0.0
            return None;
        }

        // Note that the formula in the graphics codex misses this negative sign
        let t = -(ray.origin() - self.point).dot(self.normal) / dot_dir_normal;
        if !t_range.contains(&t) {
            return None;
        }

        Some(RayIntersection {
            ray_parameter: t,
            hit_point: ray.at(t),
            normal: self.normal,
        })
    }
}
