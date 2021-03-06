use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaneSide {
    /// In front of the plane (or on its face)
    Front,
    /// Behind the plane
    Back,
}

/// A flat, two-sided, infinite plane
#[derive(Debug, Clone, PartialEq)]
pub struct InfinitePlane {
    /// The normal of the plane (MUST be a unit vector)
    ///
    /// Assumption: In order to be visible, the normal must point *towards* the view
    pub normal: Vec3,
    /// Any point on the plane
    pub point: Vec3,
}

impl InfinitePlane {
    /// Returns which side of this place the given point is on.
    pub fn which_side(&self, other_point: Vec3) -> PlaneSide {
        // Need to compare with 0.0, not EPSILON or else ray_hit_axis_aligned_plane will not always
        // produce a solution.
        if (other_point - self.point).dot(self.normal) >= 0.0 {
            PlaneSide::Front
        } else {
            PlaneSide::Back
        }
    }

    /// Returns a plane with the normal flipped
    pub fn flipped(&self) -> Self {
        Self {
            normal: -self.normal,
            point: self.point,
        }
    }
}

impl RayHit for InfinitePlane {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // Formula provided in the Graphics Codex, Ray Casting Chapter, Section 3
        // http://graphicscodex.com
        // Can be derived by substituting ray into implicit plane equation.

        // Four cases where intersection can fail
        // 1. n.dot(d) == 0 (plane is perpendicular to the normal / parallel with the plane)
        // 2. (origin - point).dot(n) == 0 (ray is entirely in the plane)
        // 3. t < 0 (intersected in the past)
        // 4. n.dot(d) > 0 (intersection with the back of the plane face)
        //
        // 1 and 4 are checked for directly in the next step. 2 and 3 are caught by checking if t
        // is in t_range since t_range is typically (EPSILON, some positive value).
        //
        // UPDATE: Case 4 is now accepted as valid in order to support refracted rays leaving the
        //  inside of a surface. That means that all planes are two-sided.

        let dot_dir_normal = ray.direction().dot(self.normal);

        // Note that the formula in the graphics codex misses this negative sign
        let t = -(ray.origin() - self.point).dot(self.normal) / dot_dir_normal;
        if !t_range.contains(&t) {
            return None;
        }

        Some(RayIntersection {
            ray_parameter: t,
            hit_point: ray.at(t),
            normal: self.normal,
            tex_coord: None,
            normal_map_transform: None,
        })
    }
}
