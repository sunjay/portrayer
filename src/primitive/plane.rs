use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Vec3};

/// The axis-aligned direction of the normal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneNormal {
    /// +x axis
    ToRight,
    /// -x axis
    ToLeft,
    /// +y axis
    ToTop,
    /// -y axis
    ToBottom,
    /// +z axis
    ToNear,
    /// -z axis
    ToFar,
}

impl PlaneNormal {
    /// Returns the normal vector corresponding to this direction
    pub fn to_vec(self) -> Vec3 {
        use PlaneNormal::*;
        match self {
            ToRight => Vec3 {x: 1.0, y: 0.0, z: 0.0},
            ToLeft => Vec3 {x: -1.0, y: 0.0, z: 0.0},
            ToTop => Vec3 {x: 0.0, y: 1.0, z: 0.0},
            ToBottom => Vec3 {x: 0.0, y: -1.0, z: 0.0},
            ToNear => Vec3 {x: 0.0, y: 0.0, z: 1.0},
            ToFar => Vec3 {x: 0.0, y: 0.0, z: -1.0},
        }
    }
}

/// A flat, axis-aligned plane with center (0,0,0) and a width and length of 1.0.
///
/// The plane faces in the direction of the normal.
#[derive(Debug)]
pub struct Plane {
    /// Any point on the plane (typically within the width and length)
    pub point: Vec3,
    /// The axis-aligned normal of the plane
    ///
    /// Assumption: In order to be visible, the normal must point *towards* the view
    pub normal: PlaneNormal,
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

        let normal = self.normal.to_vec();
        let dot_dir_normal = ray_dir.dot(normal);
        if dot_dir_normal >= -EPSILON { // >= 0.0
            return None;
        }

        let t = (ray.origin() - self.point).dot(normal) / dot_dir_normal;
        if !t_range.contains(&t) {
            return None;
        }

        let hit_point = ray.at(t);
        // At this point, we already know that there is a hit point that lies *on* the plane. Now
        // we must figure out if it is within the plane's dimensions. Notice that because the point
        // is already on the plane, it doesn't matter that the check below would work for the
        // entire volume of a cube the same size as the plane.
        let Vec3 {x, y, z} = hit_point;
        if -0.5 <= x && x <= 0.5 && -0.5 <= y && y <= 0.5 && -0.5 <= z && z <= 0.5 {
            Some(RayIntersection {
                ray_parameter: t,
                hit_point,
                normal: normal,
            })
        } else {
            None
        }
    }
}
