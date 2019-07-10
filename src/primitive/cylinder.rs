use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{Vec3, Quadratic};
use crate::bounding_box::{BoundingBox, Bounds};

/// The radius of the cylinder
const RADIUS: f64 = 1.0;
const HEIGHT: f64 = 1.0;
const HALF_HEIGHT: f64 = HEIGHT / 2.0;

/// A sphere with center (0, 0, 0), radius = 1.0, and height = 1.0
///
/// It is expected that this sphere will be used via affine transformations on the node that
/// contains it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cylinder;

impl Bounds for Cylinder {
    fn bounds(&self) -> BoundingBox {
        let min = Vec3 {x: -RADIUS, y: -HALF_HEIGHT, z: -RADIUS};
        let max = Vec3 {x: RADIUS, y: HALF_HEIGHT, z: RADIUS};
        BoundingBox::new(min, max)
    }
}

impl RayHit for Cylinder {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // Equation for a cylinder: x^2 + z^2 = r^2
        // Ray equation: r(t) = p + td
        //
        // Finding intersection: (substitute r.x and r.z into x and z in cylinder equation)
        //     (p.x + t*d.x)^2 + (p.z + t*d.z)^2 = r^2
        //     p.x^2 + 2*t*p.x*d.x + t^2*d.x^2 + p.z^2 + 2*t*p.z*d.z + t^2*d.z^2 - r^2 = 0
        //     (d.x^2 + d.z^2)*t^2 + (2*p.x*d.x + 2*p.z*d.z)*t + (p.x^2 + p.z^2 - r^2) = 0
        //
        // Solving this will give us whether the ray intersects with any side of an infinitely
        // long cylinder. To cap the cylinder, we limit the y value to be in [-height/2, height/2].
        let origin = ray.origin();
        let direction = ray.direction();

        // Suppose ax^2 + bx + c = 0, then a, b, and c for the above equation map to:
        let a = direction.x*direction.x + direction.z*direction.z;
        let b = 2.0*origin.x*direction.x + 2.0*origin.z*direction.z;
        let c = origin.x*origin.x + origin.z*origin.z - RADIUS*RADIUS;

        let equation = Quadratic {a, b, c};
        // Solve the equation and filter out any solutions not in the accepted range. This saves
        // us from having to do the check over and over again later.
        let mut t = equation.solve().filter(|sol| t_range.contains(sol));

        // There can be up to two solutions to this quadratic equation
        let t0 = t.next();
        let t1 = t.next();

        let t = match (t0, t1) {
            // No solution => no intersection
            (None, None) => return None,
            // Only one solution => intersection tangent to cylinder
            (Some(t), None) => t,
            (Some(t0), Some(t1)) => t0,
            (None, Some(_)) => unreachable!("iterator should not produce a value after it returns None"),
        };

        let hit_point = ray.at(t);
        // Normal is just the hit point - the center at the same height (y value) as the hit point
        // Since the center is (0,0,0), this is the same as just setting the y value to zero.
        let normal = Vec3 {y: 0.0, ..hit_point};

        Some(RayIntersection {
            ray_parameter: t,
            hit_point,
            normal,
            tex_coord: None,
        })
    }
}
