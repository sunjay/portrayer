use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Vec3};

/// A surface containing a single hole, shaped like a donut
///
/// More Info: http://mathworld.wolfram.com/Torus.html
#[derive(Debug)]
pub struct Torus {
    /// The radius from the center of the hole to the center of the torus tube
    center_radius: f64,
    /// The radius of the tube
    tube_radius: f64,
}

impl RayHit for Torus {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // Equations from: http://mathworld.wolfram.com/Torus.html
        // Derivation completed by Sunjay Varma (with help from our computer overlords)

        // Torus equation in Cartesian coordinates: (changed to be symmetrical about y-axis)
        //     (c - sqrt(x^2 + z^2))^2 + y^2 = a^2  where c = center_radius and a = tube_radius
        //
        // Ray Equation: r(t) = p + t*d
        //     This equation can be broken up into its x,y,z components.
        //
        // Expanding:
        //     (c - sqrt(x^2 + z^2))^2 + y^2 = a^2
        //     c^2 - 2*c*sqrt(x^2 + z^2) + x^2 + z^2 + y^2 = a^2
        //
        // Substituting:
        //     c^2 - 2*c*sqrt((p.x + t*d.x)^2 + (p.z + t*d.z)^2) + (p.x + t*d.x)^2 + (p.z + t*d.z)^2 + (p.y + t*d.y)^2 = a^2
        // Rearranging:
        //     c^2 + (p.x + t*d.x)^2 + (p.z + t*d.z)^2 + (p.y + t*d.y)^2 - a^2 = 2*c*sqrt((p.x + t*d.x)^2 + (p.z + t*d.z)^2)
        // Squaring both sides and expanding: (via Wolfram Alpha)
        //     (c^2 + (p.x + t*d.x)^2 + (p.z + t*d.z)^2 + (p.y + t*d.y)^2 - a^2)^2 - (2*c*sqrt((p.x + t*d.x)^2 + (p.z + t*d.z)^2))^2 = 0
        //
        // Grouping previous step answer by t: (via Python)
        //     t^4 : d.x^4 + 2*d.x^2*d.y^2 + 2*d.x^2*d.z^2 + d.y^4 + 2*d.y^2*d.z^2 + d.z^4
        //     t^3 : 4*d.x^3*p.x + 4*d.x^2*d.y*p.y + 4*d.x^2*d.z*p.z + 4*d.x*d.y^2*p.x + 4*d.x*d.z^2*p.x + 4*d.y^3*p.y + 4*d.y^2*d.z*p.z + 4*d.y*d.z^2*p.y + 4*d.z^3*p.z
        //     t^2 : -2*a^2*d.x^2 - 2*a^2*d.y^2 - 2*a^2*d.z^2 - 2*c^2*d.x^2 + 2*c^2*d.y^2 - 2*c^2*d.z^2 + 6*d.x^2*p.x^2 + 2*d.x^2*p.y^2 + 2*d.x^2*p.z^2 + 8*d.x*d.y*p.x*p.y + 8*d.x*d.z*p.x*p.z + 2*d.y^2*p.x^2 + 6*d.y^2*p.y^2 + 2*d.y^2*p.z^2 + 8*d.y*d.z*p.y*p.z + 2*d.z^2*p.x^2 + 2*d.z^2*p.y^2 + 6*d.z^2*p.z^2
        //     t^1 : -4*a^2*d.x*p.x - 4*a^2*d.y*p.y - 4*a^2*d.z*p.z - 4*c^2*d.x*p.x + 4*c^2*d.y*p.y - 4*c^2*d.z*p.z + 4*d.x*p.x^3 + 4*d.x*p.x*p.y^2 + 4*d.x*p.x*p.z^2 + 4*d.y*p.x^2*p.y + 4*d.y*p.y^3 + 4*d.y*p.y*p.z^2 + 4*d.z*p.x^2*p.z + 4*d.z*p.y^2*p.z + 4*d.z*p.z^3
        //     t^0 : a^4 - 2*a^2*c^2 - 2*a^2*p.x^2 - 2*a^2*p.y^2 - 2*a^2*p.z^2 + c^4 - 2*c^2*p.x^2 + 2*c^2*p.y^2 - 2*c^2*p.z^2 + p.x^4 + 2*p.x^2*p.y^2 + 2*p.x^2*p.z^2 + p.y^4 + 2*p.y^2*p.z^2 + p.z^4
        //
        // Factoring: (via Wolfram Alpha + By Hand)
        //     t^4 : (d.x^2 + d.y^2 + d.z^2)^2
        //     t^3 : 4*(d.x^2 + d.y^2 + d.z^2)*(d.x*p.x + d.y*p.y + d.z*p.z)
        //     t^2 : 2*(d.x^2 + d.y^2 + d.z^2)*(p.x^2 + p.y^2 + p.z^2 - (c^2 + a^2)) + 4(p.x*d.x + p.y*d.y + p.z*d.z)^2 + 4*c^2*d.y^2
        //     t^1 : 4*(p.x^2 + p.y^2 + p.z^2 - (a^2 + c^2))*(p.x*d.x + p.y*d.y + p.z*d.z) + 8*c^2*p.y*d.y
        //     t^0 : (p.x^2 + p.y^2 + p.z^2 - (a^2 + c^2))^2 - 4*c^2(a^2-p.y^2)
        //
        // Checked against this blog post: https://marcin-chwedczuk.github.io/ray-tracing-torus
        unimplemented!()
    }
}
