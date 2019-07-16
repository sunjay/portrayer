use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::Quartic;

/// A surface containing a single hole, shaped like a donut.
///
/// The torus has center (0,0,0) and is oriented so that the y-axis passes straight through the
/// hole.
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
        //     t^2 : 2*(d.x^2 + d.y^2 + d.z^2)*(p.x^2 + p.y^2 + p.z^2 - (a^2 + c^2)) + 4*(p.x*d.x + p.y*d.y + p.z*d.z)^2 + 4*c^2*d.y^2
        //     t^1 : 4*(p.x^2 + p.y^2 + p.z^2 - (a^2 + c^2))*(p.x*d.x + p.y*d.y + p.z*d.z) + 8*c^2*p.y*d.y
        //     t^0 : (p.x^2 + p.y^2 + p.z^2 - (a^2 + c^2))^2 - 4*c^2*(a^2 - p.y^2)
        //
        // In terms of vector operations: (. = dot product)
        //     t^4 : (d . d)*(d . d)
        //     t^3 : 4*(d . d)*(d . p)
        //     t^2 : 2*(d . d)*((p . p) - (a^2 + c^2)) + 4*(d . p)^2 + 4*c^2*d.y^2
        //     t^1 : 4*((p . p) - (a^2 + c^2))*(d . p) + 2*4*c^2*p.y*d.y
        //     t^0 : ((p . p) - (a^2 + c^2))^2 - 4*c^2*(a^2 - p.y^2)
        //
        // Checked against this blog post: https://marcin-chwedczuk.github.io/ray-tracing-torus
        //
        // These equations give the 5 constants of a quartic equation:
        //     a*t^4 + b*t^3 + c*t^2 + d*t + e = 0

        let origin = ray.origin();
        let direction = ray.direction();

        // d_dot_d = d.x*d.x + d.y*d.y + d.z*d.z
        let d_dot_d = direction.dot(direction);
        // p_dot_p = p.x*p.x + p.y*p.y + p.z*p.z
        let p_dot_p = origin.dot(origin);
        // d_dot_p = d.x*p.x + d.y*p.y + d.z*p.z
        let d_dot_p = direction.dot(origin);

        let Self {tube_radius, center_radius} = *self;

        // a^2
        let a_sqr = tube_radius * tube_radius;
        // c^2
        let c_sqr = center_radius * center_radius;
        // a^2 + c^2
        let radii_sqr = a_sqr + c_sqr;
        // (p . p) - (a^2 + c^2)
        let p_dot_p_minus_radii_sqr = p_dot_p - radii_sqr;
        // 4*c^2
        let four_c_sqr = 4.0 * c_sqr;
        // p.y^2
        let p_y_sqr = origin.y * origin.y;
        // d.y^2
        let d_y_sqr = direction.y * direction.y;

        // Compute quartic coefficients
        let a = d_dot_d*d_dot_d;
        let b = 4.0*d_dot_d*d_dot_p;
        let c = 2.0*d_dot_d*p_dot_p_minus_radii_sqr + 4.0*d_dot_p*d_dot_p + four_c_sqr*d_y_sqr;
        let d = 4.0*p_dot_p_minus_radii_sqr*d_dot_p + 2.0*four_c_sqr*origin.y*direction.y;
        let e = p_dot_p_minus_radii_sqr*p_dot_p_minus_radii_sqr - four_c_sqr*(a_sqr - p_y_sqr);

        let equation = Quartic {a, b, c, d, e};
        let t = equation.solve().find_in_range(t_range)?;

        let hit_point = ray.at(t);

        // One way to find the normal is to find a point at the center of the tube nearest to the
        // hit_point and use:
        //     hit_point - tube_center
        // This will give you a vector perpendicular to the surface at hit_point.
        //
        // We know that:
        //   * the center of the tube, tube_center, is on a circle given by: x^2 + z^2 = c^2
        //   * the distance between the hit_point and the tube_center is: a = tube_radius
        //     (hit_point - tube_center) . (hit_point - tube_center) = a^2
        //
        // Suppose tube_center = (xc, 0.0, zc) and hit_point = (x_hit, y_hit, z_hit)
        // This gives us:
        //     (x_hit - xc)^2 + (y_hit - 0.0)^2 + (z_hit - zc)^2 = a^2       (1)
        //     xc^2 + zc^2 = c^2                                             (2)
        
        Some(RayIntersection {
            ray_parameter: t,
            hit_point,
            normal: unimplemented!(),
            tex_coord: None,
            normal_map_transform: None,
        })
    }
}
