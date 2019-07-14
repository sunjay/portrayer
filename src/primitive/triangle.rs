use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{Vec3, Uv};
use crate::bounding_box::{BoundingBox, Bounds};

/// A triangle with the given 3 vertices
#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
    /// The normals for a, b, and c respectively. If not provided, the intersection
    /// normal will be the same for all points on the triangle.
    pub normals: Option<(Vec3, Vec3, Vec3)>,
    /// The texture coordinates for a, b, and c respectively. The texture coordinate
    /// for a ray hit will only be set if these are provided.
    pub tex_coords: Option<(Uv, Uv, Uv)>
}

impl Triangle {
    /// Creates a new flat shaded triangle. Normals will be computed from the given
    /// vertices and will be same all across the face.
    pub fn flat(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self {a, b, c, normals: None, tex_coords: None}
    }
}

impl Bounds for Triangle {
    fn bounds(&self) -> BoundingBox {
        let Triangle {a, b, c, ..} = *self;
        let min = Vec3::partial_min(a, Vec3::partial_min(b, c));
        let max = Vec3::partial_max(a, Vec3::partial_max(b, c));
        BoundingBox::new(min, max)
    }
}

impl RayHit for Triangle {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // Full formulas provided in Peter Shirley's ray tracing chapter (pg 208)
        // http://www.cs.utah.edu/~shirley/books/fcg2/rt.pdf
        // Can be derived using Cramer's rule

        // "A" matrix (LHS)

        let Vec3 {x: a, y: b, z: c} = self.a - self.b;
        let Vec3 {x: d, y: e, z: f} = self.a - self.c;
        let Vec3 {x: g, y: h, z: i} = ray.direction();

        // "R" matrix (RHS)

        let Vec3 {x: j, y: k, z: l} = self.a - ray.origin();

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

        let normal = match self.normals {
            Some((na, nb, nc)) => {
                let alpha = 1.0 - beta - gamma;
                na * alpha + nb * beta + nc * gamma
            },
            None => (self.b - self.a).cross(self.c - self.a),
        };

        let tex_coord = match self.tex_coords {
            Some((ta, tb, tc)) => {
                let alpha = 1.0 - beta - gamma;
                let uv = ta * alpha + tb * beta + tc * gamma;
                // Need to reverse uv because we've been using a top-to-bottom convention where the
                // rest of the world uses a bottom to top convention
                //TODO: Consider reversing this everywhere else in the code instead so that we
                // follow the rest of the world in our UV coordinate conventions
                Some(Uv {u: uv.u, v: 1.0 - uv.v})
            },
            None => None,
        };

        Some(RayIntersection {
            ray_parameter: t,
            hit_point: ray.at(t),
            normal,
            tex_coord,
            normal_map_transform: None,
        })
    }
}
