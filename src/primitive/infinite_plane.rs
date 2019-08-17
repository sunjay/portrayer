use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaneSide {
    /// In front of the plane (or on its face)
    Front,
    /// Behind the plane
    Back,
}

/// A flat, two-sided, axis-aligned, infinite plane
#[derive(Debug, Clone, PartialEq)]
pub enum InfinitePlane {
    Up(InfinitePlaneUp),
    Down(InfinitePlaneDown),
    Right(InfinitePlaneRight),
    Left(InfinitePlaneLeft),
    Front(InfinitePlaneFront),
    Back(InfinitePlaneBack),
}

impl InfinitePlane {
    /// Returns the point on the plane
    pub fn point(&self) -> Vec3 {
        use InfinitePlane::*;
        match self {
            Up(plane) => plane.point,
            Down(plane) => plane.point,
            Right(plane) => plane.point,
            Left(plane) => plane.point,
            Front(plane) => plane.point,
            Back(plane) => plane.point,
        }
    }

    /// Returns the point on the plane
    pub fn point_mut(&mut self) -> &mut Vec3 {
        use InfinitePlane::*;
        match self {
            Up(plane) => &mut plane.point,
            Down(plane) => &mut plane.point,
            Right(plane) => &mut plane.point,
            Left(plane) => &mut plane.point,
            Front(plane) => &mut plane.point,
            Back(plane) => &mut plane.point,
        }
    }

    /// Returns the normal vector of this plane
    pub fn normal(&self) -> Vec3 {
        use InfinitePlane::*;
        match self {
            Up(plane) => plane.normal(),
            Down(plane) => plane.normal(),
            Right(plane) => plane.normal(),
            Left(plane) => plane.normal(),
            Front(plane) => plane.normal(),
            Back(plane) => plane.normal(),
        }
    }

    /// Returns which side of this plane the given point is on.
    pub fn which_side(&self, other_point: Vec3) -> PlaneSide {
        use InfinitePlane::*;
        match self {
            Up(plane) => plane.which_side(other_point),
            Down(plane) => plane.which_side(other_point),
            Right(plane) => plane.which_side(other_point),
            Left(plane) => plane.which_side(other_point),
            Front(plane) => plane.which_side(other_point),
            Back(plane) => plane.which_side(other_point),
        }
    }
}

impl RayHit for InfinitePlane {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        use InfinitePlane::*;
        match self {
            Up(plane) => plane.ray_hit(ray, t_range),
            Down(plane) => plane.ray_hit(ray, t_range),
            Right(plane) => plane.ray_hit(ray, t_range),
            Left(plane) => plane.ray_hit(ray, t_range),
            Front(plane) => plane.ray_hit(ray, t_range),
            Back(plane) => plane.ray_hit(ray, t_range),
        }
    }
}

// Generates an infinite plane primitive. The normal must be such that the plane is axis-aligned.
//
// * normal_sign must be 1.0 or -1.0 and must match whether the normal is positive or negative.
// * normal_axis is an ident (either x, y, or z) representing the non-zero component of the normal.
macro_rules! infinite_plane {
    (
        doc: $doc:expr,
        name: $name:ident,
        variant: $variant:ident,
        normal: $normal:expr,
        normal_sign: $normal_sign:expr,
        normal_axis: $axis:ident,
    ) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            /// Any point on the plane
            pub point: Vec3,
        }

        impl From<$name> for InfinitePlane {
            fn from(plane: $name) -> Self {
                InfinitePlane::$variant(plane)
            }
        }

        impl $name {
            /// Returns the normal vector of this plane
            pub fn normal(&self) -> Vec3 {
                $normal
            }

            /// Returns which side of this plane the given point is on.
            pub fn which_side(&self, other_point: Vec3) -> PlaneSide {
                if $normal_sign * other_point.$axis >= $normal_sign * self.point.$axis {
                    PlaneSide::Front
                } else {
                    PlaneSide::Back
                }
            }
        }

        impl RayHit for $name {
            fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
                // Since the plane is axis-aligned, we know that all of the coordinates on the
                // plane along one of the X, Y, or Z axes will be same. We can take advantage of
                // this to very efficiently find the intersection. It is important to be very
                // careful here though because if the ray and the plane are parallel to each other,
                // there is no intersection.
                //
                // Here is the derivation for a plane where all points have the same y-coordinate:
                //
                //     Ray equation: r(t) = p + t*d
                //     Point on plane: (x, y, z)
                //
                //     y = p.y + t*d.y
                //     t = (y - p.y) / d.y
                //
                // If d.y == 0, then the plane and the ray are parallel and there is a danger that
                // we may divide by zero.

                let direction = ray.direction();
                if direction.$axis.abs() < EPSILON {
                    // Plane and ray are parallel, no intersection
                    return None;
                }

                let t = (self.point.$axis - ray.origin().$axis) / direction.$axis;
                if !t_range.contains(&t) {
                    return None;
                }

                Some(RayIntersection {
                    ray_parameter: t,
                    hit_point: ray.at(t),
                    normal: self.normal(),
                    tex_coord: None,
                    normal_map_transform: None,
                })
            }
        }
    };
}

infinite_plane!(
    doc: "A flat, two-sided, axis-aligned, infinite plane where the normal faces up (0,1,0)",
    name: InfinitePlaneUp,
    variant: Up,
    normal: Vec3::up(),
    normal_sign: 1.0,
    normal_axis: y,
);

infinite_plane!(
    doc: "A flat, two-sided, axis-aligned, infinite plane where the normal faces down (0,-1,0)",
    name: InfinitePlaneDown,
    variant: Down,
    normal: Vec3::down(),
    normal_sign: -1.0,
    normal_axis: y,
);

infinite_plane!(
    doc: "A flat, two-sided, axis-aligned, infinite plane where the normal faces right (1,0,0)",
    name: InfinitePlaneRight,
    variant: Right,
    normal: Vec3::right(),
    normal_sign: 1.0,
    normal_axis: x,
);

infinite_plane!(
    doc: "A flat, two-sided, axis-aligned, infinite plane where the normal faces left (-1,0,0)",
    name: InfinitePlaneLeft,
    variant: Left,
    normal: Vec3::left(),
    normal_sign: -1.0,
    normal_axis: x,
);

infinite_plane!(
    doc: "A flat, two-sided, axis-aligned, infinite plane where the normal faces the front (0,0,1)",
    name: InfinitePlaneFront,
    variant: Front,
    normal: Vec3::back_rh(),
    normal_sign: 1.0,
    normal_axis: z,
);

infinite_plane!(
    doc: "A flat, two-sided, axis-aligned, infinite plane where the normal faces the back (0,0,-1)",
    name: InfinitePlaneBack,
    variant: Back,
    normal: Vec3::forward_rh(),
    normal_sign: -1.0,
    normal_axis: z,
);
