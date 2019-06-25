use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::Quadratic;

/// A sphere with center (0, 0, 0) and radius 1.0
///
/// It is expected that this sphere will be used via affine transformations on the node that
/// contains it.
#[derive(Debug)]
pub struct Sphere;

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

        let a = ray.direction().dot(ray.direction());
        let b = 2.0 * ray.origin().dot(ray.direction());
        let c = ray.origin().dot(ray.origin()) - 1.0;

        let equation = Quadratic {a, b, c};
        let t = equation.solve().find(|sol| t_range.contains(sol))?;

        let hit_point = ray.at(t);
        Some(RayIntersection {
            ray_parameter: t,
            hit_point,
            // Normal of sphere is the hit point on the sphere - the center (0, 0, 0)
            // Note that we do divide by the radius because the radius is 1.0
            normal: hit_point,
        })
    }
}
