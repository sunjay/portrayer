use std::f64::consts::PI;
use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{Vec3, Quadratic, Uv};
use crate::bounding_box::{BoundingBox, Bounds};

/// The radius of the sphere
const RADIUS: f64 = 1.0;

/// A sphere with center (0, 0, 0) and radius 1.0
///
/// It is expected that this sphere will be used via affine transformations on the node that
/// contains it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sphere;

impl Bounds for Sphere {
    fn bounds(&self) -> BoundingBox {
        let min = Vec3::from(-RADIUS);
        let max = Vec3::from(RADIUS);
        BoundingBox::new(min, max)
    }
}

impl RayHit for Sphere {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // Equation for sphere: x*x + y*y + z*z = R*R
        // Equation for sphere with center (0,0,0) and radius 1.0:
        //     x*x + y*y + z*z = 1.0
        // If p = (x, y, z), then this can be written:
        //     p.dot(p) = 1.0
        // The ray has equation:
        //     r = o + t*d
        // To find the intersection, let p = r
        //     (o + t*d).dot(o + t*d) = 1.0
        // Expanding:
        //     t*t*d.dot(d) 2*t*d.dot(o) + o.dot(o) - 1.0 = 0
        //
        // This quadratic equation (a*t^2 + b*t + c = 0) can be solved using the quadratic formula.
        //
        // The discriminant b*b - 4*a*c tells us the number of solutions. A positive discriminant
        // (two solutions) means that the sphere was intersected by the ray twice. If the
        // discriminant is equal to zero, there is just a single intersection on the edge of the
        // sphere. If the discriminant is negative, no intersection occurred and we can return
        // early.

        let origin = ray.origin();
        let direction = ray.direction();

        let a = direction.dot(direction);
        let b = 2.0 * origin.dot(direction);
        let c = origin.dot(origin) - RADIUS * RADIUS;

        let equation = Quadratic {a, b, c};
        let t = equation.solve().find(|sol| t_range.contains(sol))?;

        let hit_point = ray.at(t);
        Some(RayIntersection {
            ray_parameter: t,
            hit_point,
            // Normal of sphere is the hit point on the sphere - the center (0, 0, 0)
            // Note that we do divide by the radius because the radius is 1.0
            normal: hit_point,
            tex_coord: Some(Uv {
                // Using spherical coordinates.
                // Formula from Fundamentals of Computer Graphics, 4th ed. Chapter 11.2.1
                // The addition/subtraction and the division maps the angles to the 0.0 to 1.0 range
                // Signs of x, y, z adjusted to account for axis convention
                u: (PI + (-hit_point.z).atan2(hit_point.x)) / (2.0 * PI),
                v: hit_point.y.acos() / PI,
            }),
        })
    }
}
