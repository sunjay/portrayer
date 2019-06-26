use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::Vec3;

/// A triangle with the given 3 vertices
#[derive(Debug)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl RayHit for Triangle {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // Full formulas provided in Peter Shirley's ray tracing chapter (pg 208)
        // http://www.cs.utah.edu/~shirley/books/fcg2/rt.pdf
        // Can be derived using Cramer's rule

        // "A" matrix (LHS)

        let a = self.a.x - self.b.x;
        let b = self.a.y - self.b.y;
        let c = self.a.z - self.b.z;

        let d = self.a.x - self.c.x;
        let e = self.a.y - self.c.y;
        let f = self.a.z - self.c.z;

        let Vec3 {x: g, y: h, z: i} = ray.direction();

        // "R" matrix (RHS)

        let origin = ray.origin();
        let j = self.a.x - origin.x;
        let k = self.a.y - origin.y;
        let l = self.a.z - origin.z;

        // "M" calculation
        let ei_hf = e*i - h*f;
        let gf_di = g*f - d*i;
        let dh_eg = d*h - e*g;
        let m = a*ei_hf + b*gf_di + c*dh_eg;

        // Calculate "t"

        let ak_jb = a*k - j*b;
        let jc_al = j*c - a*l;
        let bl_ck = b*l - c*k;

        let t = -(f * ak_jb + e * jc_al + d * bl_ck) / m;
        if !t_range.contains(&t) {
            return None;
        }

        let gamma = (i * ak_jb + h * jc_al + g * bl_ck) / m;
        if gamma < 0.0 || gamma > 1.0 {
            return None;
        }

        let beta = (j*ei_hf + k*gf_di + l*dh_eg) / m;
        if beta < 0.0 || beta > 1.0 - gamma {
            return None;
        }

        Some(RayIntersection {
            ray_parameter: t,
            hit_point: ray.at(t),
            normal: (self.b - self.a).cross(self.c - self.a),
        })
    }
}
