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
        // Derivation completed by Sunjay Varma

        // Torus equation in Cartesian coordinates: (changed to be symmetrical about y-axis)
        //     (c - sqrt(x^2 + z^2))^2 + y^2 = a^2  where c = center_radius and a = tube_radius
        //
        // The derivation is easier if we break this equation up into two equations:
        //     w^2 = x^2 + z^2            (1)
        //     (c - w)^2 + y^2 = a^2      (2)
        //
        // Then we can derive equations for t for both of these cases and solve the system.
        //
        // Ray Equation: r(t) = p + t*d
        //     This equation can be broken up into its x,y,z components.
        //
        // For equation (1):
        //     w^2 = (p.x + t*d.x)^2 + (p.z + t*d.z)^2
        //     w^2 = p.x^2 + 2*t*p.x*d.x + t^2*d.x^2 + p.z^2 + 2*t*p.z*d.z + t^2*d.z^2
        //     0 = t^2*d.x^2 + t^2*d.z^2 + 2*t*p.x*d.x + 2*t*p.z*d.z + p.x^2 + p.z^2 - w^2
        //     0 = t^2*(d.x^2 + d.z^2) + t*(2*p.x*d.x + 2*p.z*d.z) + (p.x^2 + p.z^2 - w^2)
        //
        // For equation (2):
        //     (c - w)^2 + (p.y + t*d.y)^2 = a^2
        //     c^2 - 2*c*w + w^2 + p.y^2 + 2*t*p.y*d.y + t^2*d.y^2 = a^2
        //     w^2 - 2*c*w + p.y^2 + 2*t*p.y*d.y + t^2*d.y^2 + c^2 - a^2 = 0

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
        //     t^2 : -2*a^2*d.x^2 + -2*a^2*d.y^2 + -2*a^2*d.z^2 + -2*c^2*d.x^2 + 2*c^2*d.y^2 + -2*c^2*d.z^2 + 6*d.x^2*p.x^2 + 2*d.x^2*p.y^2 + 2*d.x^2*p.z^2 + 8*d.x*d.y*p.x*p.y + 8*d.x*d.z*p.x*p.z + 2*d.y^2*p.x^2 + 6*d.y^2*p.y^2 + 2*d.y^2*p.z^2 + 8*d.y*d.z*p.y*p.z + 2*d.z^2*p.x^2 + 2*d.z^2*p.y^2 + 6*d.z^2*p.z^2
        //     t^1 : -4*a^2*d.x*p.x + -4*a^2*d.y*p.y + -4*a^2*d.z*p.z + -4*c^2*d.x*p.x + 4*c^2*d.y*p.y + -4*c^2*d.z*p.z + 4*d.x*p.x^3 + 4*d.x*p.x*p.y^2 + 4*d.x*p.x*p.z^2 + 4*d.y*p.x^2*p.y + 4*d.y*p.y^3 + 4*d.y*p.y*p.z^2 + 4*d.z*p.x^2*p.z + 4*d.z*p.y^2*p.z + 4*d.z*p.z^3
        //     t^0 : a^4 + -2*a^2*c^2 + -2*a^2*p.x^2 + -2*a^2*p.y^2 + -2*a^2*p.z^2 + c^4 + -2*c^2*p.x^2 + 2*c^2*p.y^2 + -2*c^2*p.z^2 + p.x^4 + 2*p.x^2*p.y^2 + 2*p.x^2*p.z^2 + p.y^4 + 2*p.y^2*p.z^2 + p.z^4
        //
        // Factoring: (via Wolfram Alpha)
        //     t^4 : (d.x^2 + d.y^2 + d.z^2)^2
        //     t^3 : 4*(d.x^2 + d.y^2 + d.z^2)*(d.x*p.x + d.y*p.y + d.z*p.z)
        //     t^2 :
        //
        //     t^4 : d1^4 + 2*d1^2*d2^2 + 2*d1^2*d3^2 + d2^4 + 2*d2^2*d3^2 + d3^4
        //     t^3 : 4*d1^3*p1 + 4*d1^2*d2*p2 + 4*d1^2*d3*p3 + 4*d1*d2^2*p1 + 4*d1*d3^2*p1 + 4*d2^3*p2 + 4*d2^2*d3*p3 + 4*d2*d3^2*p2 + 4*d3^3*p3
        //     t^2 : -2*a^2*d1^2 + -2*a^2*d2^2 + -2*a^2*d3^2 + -2*c^2*d1^2 + 2*c^2*d2^2 + -2*c^2*d3^2 + 6*d1^2*p1^2 + 2*d1^2*p2^2 + 2*d1^2*p3^2 + 8*d1*d2*p1*p2 + 8*d1*d3*p1*p3 + 2*d2^2*p1^2 + 6*d2^2*p2^2 + 2*d2^2*p3^2 + 8*d2*d3*p2*p3 + 2*d3^2*p1^2 + 2*d3^2*p2^2 + 6*d3^2*p3^2
        //     t^1 : -4*a^2*d1*p1 + -4*a^2*d2*p2 + -4*a^2*d3*p3 + -4*c^2*d1*p1 + 4*c^2*d2*p2 + -4*c^2*d3*p3 + 4*d1*p1^3 + 4*d1*p1*p2^2 + 4*d1*p1*p3^2 + 4*d2*p1^2*p2 + 4*d2*p2^3 + 4*d2*p2*p3^2 + 4*d3*p1^2*p3 + 4*d3*p2^2*p3 + 4*d3*p3^3
        //     t^0 : a^4 + -2*a^2*c^2 + -2*a^2*p1^2 + -2*a^2*p2^2 + -2*a^2*p3^2 + c^4 + -2*c^2*p1^2 + 2*c^2*p2^2 + -2*c^2*p3^2 + p1^4 + 2*p1^2*p2^2 + 2*p1^2*p3^2 + p2^4 + 2*p2^2*p3^2 + p3^4


        //     (c^2 + (p1 + t*d1)^2 + (p3 + t*d3)^2 + (p2 + t*d2)^2 - a^2)^2 - (2*c*sqrt((p1 + t*d1)^2 + (p3 + t*d3)^2))^2 = 0
        //
        // a^4 - 2 a^2 c^2 - 2 a^2 d1^2 t^2 - 4 a^2 d1 p1 t - 2 a^2 d2^2 t^2 - 4 a^2 d2 p2 t - 2 a^2 d3^2 t^2 - 4 a^2 d3 p3 t - 2 a^2 p1^2 - 2 a^2 p2^2 - 2 a^2 p3^2 + c^4 - 2 c^2 d1^2 t^2 - 4 c^2 d1 p1 t + 2 c^2 d2^2 t^2 + 4 c^2 d2 p2 t - 2 c^2 d3^2 t^2 - 4 c^2 d3 p3 t - 2 c^2 p1^2 + 2 c^2 p2^2 - 2 c^2 p3^2 + d1^4 t^4 + 4 d1^3 p1 t^3 + 2 d1^2 d2^2 t^4 + 4 d1^2 d2 p2 t^3 + 2 d1^2 d3^2 t^4 + 4 d1^2 d3 p3 t^3 + 6 d1^2 p1^2 t^2 + 2 d1^2 p2^2 t^2 + 2 d1^2 p3^2 t^2 + 4 d1 d2^2 p1 t^3 + 8 d1 d2 p1 p2 t^2 + 4 d1 d3^2 p1 t^3 + 8 d1 d3 p1 p3 t^2 + 4 d1 p1^3 t + 4 d1 p1 p2^2 t + 4 d1 p1 p3^2 t + d2^4 t^4 + 4 d2^3 p2 t^3 + 2 d2^2 d3^2 t^4 + 4 d2^2 d3 p3 t^3 + 2 d2^2 p1^2 t^2 + 6 d2^2 p2^2 t^2 + 2 d2^2 p3^2 t^2 + 4 d2 d3^2 p2 t^3 + 8 d2 d3 p2 p3 t^2 + 4 d2 p1^2 p2 t + 4 d2 p2^3 t + 4 d2 p2 p3^2 t + d3^4 t^4 + 4 d3^3 p3 t^3 + 2 d3^2 p1^2 t^2 + 2 d3^2 p2^2 t^2 + 6 d3^2 p3^2 t^2 + 4 d3 p1^2 p3 t + 4 d3 p2^2 p3 t + 4 d3 p3^3 t + p1^4 + 2 p1^2 p2^2 + 2 p1^2 p3^2 + p2^4 + 2 p2^2 p3^2 + p3^4
        //
        // WRONG:
        //     a^4 - 2*a^2*c^2 - 2*a^2*d.x^2*t^2 - 4*a^2*d.x*p.x*t - 2*a^2*d.y^2*t^2 - 4*a^2*d.y*p.y*t - 2*a^2*d.z^2*t^2 - 4*a^2*d.z*p.z*t - 2*a^2*p.x^2 - 2*a^2*p.y^2 - 2*a^2*p.z^2 + c^4 + 2*c^2*d.x^2*t^2 + 4*c^2*d.x*p.x*t + 2*c^2*d.y^2*t^2 + 4*c^2*d.y*p.y*t + 2*c^2*d.z^2*t^2 + 4*c^2*d.z*p.z*t + 2*c^2*p.x^2 + 2*c^2*p.y^2 + 2*c^2*p.z^2 + d.x^4*t^4 + 4*d.x^3*p.x*t^3 + 2*d.x^2*d.y^2*t^4 + 4*d.x^2*d.y*p.y*t^3 + 2*d.x^2*d.z^2*t^4 + 4*d.x^2*d.z*p.z*t^3 + 6*d.x^2*p.x^2*t^2 + 2*d.x^2*p.y^2*t^2 + 2*d.x^2*p.z^2*t^2 + 4*d.x*d.y^2*p.x*t^3 + 8*d.x*d.y*p.x*p.y*t^2 + 4*d.x*d.z^2*p.x*t^3 + 8*d.x*d.z*p.x*p.z*t^2 + 4*d.x*p.x^3*t + 4*d.x*p.x*p.y^2*t + 4*d.x*p.x*p.z^2*t + d.y^4*t^4 + 4*d.y^3*p.y*t^3 + 2*d.y^2*d.z^2*t^4 + 4*d.y^2*d.z*p.z*t^3 + 2*d.y^2*p.x^2*t^2 + 6*d.y^2*p.y^2*t^2 + 2*d.y^2*p.z^2*t^2 + 4*d.y*d.z^2*p.y*t^3 + 8*d.y*d.z*p.y*p.z*t^2 + 4*d.y*p.x^2*p.y*t + 4*d.y*p.y^3*t + 4*d.y*p.y*p.z^2*t + d.z^4*t^4 + 4*d.z^3*p.z*t^3 + 2*d.z^2*p.x^2*t^2 + 2*d.z^2*p.y^2*t^2 + 6*d.z^2*p.z^2*t^2 + 4*d.z*p.x^2*p.z*t + 4*d.z*p.y^2*p.z*t + 4*d.z*p.z^3*t + p.x^4 + 2*p.x^2*p.y^2 + 2*p.x^2*p.z^2 + p.y^4 + 2*p.y^2*p.z^2 + p.z^4*=*4*c^2*d.x^2*t^2 + 8*c^2*d.x*p.x*t + 4*c^2*d.z^2*t^2 + 8*c^2*d.z*p.z*t + 4*c^2*p.x^2 + 4*c^2*p.z^2
        //
        // Expanding: (via Wolfram Alpha)
        //     c^2 - 2*c*sqrt(d.x^2*t^2 + 2*d.x*p.x*t + d.z^2*t^2 + 2*d.z*p.z*t + p.x^2 + p.z^2) + d.x^2*t^2 + 2*d.x*p.x*t + d.y^2*t^2 + 2*d.y*p.y*t + d.z^2*t^2 + 2*d.z*p.z*t + p.x^2 + p.y^2 + p.z^2 = a^2
        // Grouping by t:
        //     d.z^2*t^2 + c^2 - 2*c*sqrt(d.x^2*t^2 + 2*d.x*p.x*t + 2*d.z*p.z*t + p.x^2 + p.z^2) + d.x^2*t^2 + 2*d.x*p.x*t + d.y^2*t^2 + 2*d.y*p.y*t + d.z^2*t^2 + 2*d.z*p.z*t + p.x^2 + p.y^2 + p.z^2 = a^2


        //
        // Parametric equations of a torus:
        //           [ x ] = [ (c + a*cos(v))*cos(u) ]
        // v(u, v) = [ y ] = [ (c + a*cos(v))*sin(u) ]
        //           [ z ] = [ a*sin(v)             ]
        //
        // Ray equation: r(t) = p + t*d
        //
        // To find the intersection, set r(t) = v(u, v) to get the three equations:
        //
        // (1)  p.x + t*d.x = (c + a*cos(v))*cos(u)
        // (2)  p.y + t*d.y = (c + a*cos(v))*sin(u)
        // (3)  p.z + t*d.z = a*sin(v)
        //
        // Three unknowns (t, u, v) and three equations.
        //
        // From (1), we can get an equation for u:
        // (4)  u = acos((p.x + t*d.x) / (c + a*cos(v)))
        // From (2), we can get another equation for u:
        // (5)  u = asin((p.y + t*d.y) / (c + a*cos(v)))
        // Then from (3), we can get an equation for v:
        // (6)  v = asin((p.z + t*d.z)/a)
        //
        // Equating (4) and (5), we get:
        // (7)  acos((p.x + t*d.x) / (c + a*cos(v))) = asin((p.y + t*d.y) / (c + a*cos(v)))
        //

        unimplemented!()
    }
}
