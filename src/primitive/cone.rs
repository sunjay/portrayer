use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{Vec3, Quadratic};
use crate::bounding_box::{BoundingBox, Bounds};

/// The radius of the cone
const RADIUS: f64 = 0.5;
const HEIGHT: f64 = 1.0;
const HALF_HEIGHT: f64 = HEIGHT / 2.0;

/// A cone with center (0, 0, 0), diameter = 1.0, and height = 1.0
///
/// It is expected that this cone will be used via affine transformations on the node that
/// contains it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cone;

impl Bounds for Cone {
    fn bounds(&self) -> BoundingBox {
        let min = Vec3 {x: -RADIUS, y: -HALF_HEIGHT, z: -RADIUS};
        let max = Vec3 {x: RADIUS, y: HALF_HEIGHT, z: RADIUS};
        BoundingBox::new(min, max)
    }
}

/// Attempt to intersect with the side of the cone
fn ray_hit_body(ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
    // Equation for a cone: x^2/R^2 + z^2/R^2 - (y - h/2)^2/h^2 = 0
    // Ray equation: r(t) = p + td
    //
    // Finding intersection: (substitute components of r into x,y,z in cone equation)
    //     (p.x + t*d.x)^2/R^2 + (p.z + t*d.z)^2/R^2 - (p.y + t*d.y - h/2)^2/h^2 = 0
    // Simplified: (via Wolfram Alpha)
    //     (-4*d.x^2*h^2*t^2 + 4*d.y^2*R^2*t^2 - 4*d.z^2*h^2*t^2 - 8*d.x*h^2*p.x*t - 4*d.y*h*R^2*t + 8*d.y*p.y*R^2*t - 8*d.z*h^2*p.z*t - 4*h^2*p.x^2 - 4*h^2*p.z^2 + h^2*R^2 - 4*h*p.y*R^2 + 4*p.y^2*R^2)/(-4*h^2*R^2) = 0
    // Multiplying both sides by (-4*h^2*R^2)
    //     -4*d.x^2*h^2*t^2 + 4*d.y^2*R^2*t^2 - 4*d.z^2*h^2*t^2 - 8*d.x*h^2*p.x*t - 4*d.y*h*R^2*t + 8*d.y*p.y*R^2*t - 8*d.z*h^2*p.z*t - 4*h^2*p.x^2 - 4*h^2*p.z^2 + h^2*R^2 - 4*h*p.y*R^2 + 4*p.y^2*R^2 = 0
    // Grouping by t:
    //     (-4*d.x^2*h^2 + 4*d.y^2*R^2 - 4*d.z^2*h^2)*t^2 + (-8*d.x*h^2*p.x - 4*d.y*h*R^2 + 8*d.y*p.y*R^2 - 8*d.z*h^2*p.z)*t + (-4*h^2*p.x^2 - 4*h^2*p.z^2 + h^2*R^2 - 4*h*p.y*R^2 + 4*p.y^2*R^2) = 0
    // For ax^2 + bx + c = 0, this gives us:
    //     a = -4*d.x^2*h^2 + 4*d.y^2*R^2 - 4*d.z^2*h^2
    //     b = -8*d.x*h^2*p.x - 4*d.y*h*R^2 + 8*d.y*p.y*R^2 - 8*d.z*h^2*p.z
    //     c = -4*h^2*p.x^2 - 4*h^2*p.z^2 + h^2*R^2 - 4*h*p.y*R^2 + 4*p.y^2*R^2
    // Factoring to lower number of floating point operations:
    //     h_sqr = h^2, R_sqr = R^2
    //     a = 4*d.y^2*R_sqr - 4*h_sqr*(d.x^2 + d.z^2)
    //     b = -8*h_sqr*(d.x*p.x + d.z*p.z) - 4*R_sqr*(d.y*h - 2*d.y*p.y)
    //     c = -4*h_sqr*(p.x^2 + p.z^2) + R_sqr*(h_sqr - 4*h*p.y + 4*p.y^2)
    //
    // Solving this will give us whether the ray intersects with any side of an infinitely long
    // cone with tip at height/2. (Note that a mirror image of the cone stretches in the other
    // direction from the tip.) To limit the cone to the right dimensions, we check the y value to
    // make sure it is in [-height/2, height/2].

    let origin = ray.origin();
    let direction = ray.direction();

    let h_sqr = HEIGHT*HEIGHT;
    let r_sqr = RADIUS*RADIUS;
    let a = 4.0*direction.y*direction.y*r_sqr - 4.0*h_sqr*(direction.x*direction.x + direction.z*direction.z);
    let b = -8.0*h_sqr*(direction.x*origin.x + direction.z*origin.z) - 4.0*r_sqr*(direction.y*HEIGHT - 2.0*direction.y*origin.y);
    let c = -4.0*h_sqr*(origin.x*origin.x + origin.z*origin.z) + r_sqr*(h_sqr - 4.0*HEIGHT*origin.y + 4.0*origin.y*origin.y);

    let equation = Quadratic {a, b, c};
    // Find the smallest t for which this equation is satisfied
    let t = equation.solve().find(|sol| t_range.contains(sol))?;
    // Stop processing as early as possible if we're not in the valid range
    if !t_range.contains(&t) {
        return None;
    }

    let hit_point = ray.at(t);
    // Test if we intersected beyond the tip or below the cap
    if hit_point.y > HALF_HEIGHT || hit_point.y < -HALF_HEIGHT {
        return None;
    }

    // For the normal, we will take advantage of the following information:
    // 1. The hit point is on the surface of the cone
    // 2. Every point on the surface is within a ring/circle centered at the y-axis
    // 3. All surface points of the cone meet at the tip: Vec3 {x: 0.0, y: HALF_HEIGHT, z: 0.0}
    // 4. Normal is perpendicular to any two non-coincident vectors tangent to the surface
    //
    // We already have one vector tangent to the surface: T1 = Tip - HitPoint
    //
    //                Tip
    //               /  \
    //   HitPoint ./     \. Opposite
    //           /        \
    //         /           \
    //        --------------
    //
    // Since the hit point is on a ring/circle, we can take advantage of symmetry to get:
    //   Opposite = {x: -HitPoint.x, y: HitPoint.y, z: -HitPoint.z}
    // We can find another tangent vector using the vector: Across = Opposite - HitPoint
    // The tangent vector T2 is thus: T2 = T1 x Across
    // With that, Normal = T1 x T2

    let tip = Vec3 {x: 0.0, y: HALF_HEIGHT, z: 0.0};
    let tangent1 = tip - hit_point;
    let opposite = Vec3 {x: -hit_point.x, y: hit_point.y, z: -hit_point.z};
    let across = opposite - hit_point;
    let tangent2 = tangent1.cross(across);
    let normal = tangent1.cross(tangent2);

    Some(RayIntersection {
        ray_parameter: t,
        hit_point,
        normal,
        tex_coord: None,
    })
}

/// Attempt to intersect with the bottom cap of the cone
fn ray_hit_cap(ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
    // An easy way to test the cap is to treat it like a plane at y = -h/2 where the intersection
    // point has to satisfy the circle equation: x^2 + z^2 <= r^2
    //
    // We don't even have to use the entire general plane intersection routine. We can exploit the
    // fact that the plane is axis-aligned and do something much cheaper. Since the plane is
    // axis-aligned to the y-axis, all intersection points must have the same y-value (`height`).
    // That means that we need only solve the y-component of the ray equation: r.y = p.y + t*d.y
    // with r.y = height. This will give us a t value. Then we can just test the x and z values
    // to see if we satisfy x^2 + z^2 <= r^2 and we're good to go.

    // Note that this solution is robust against every case:
    // 1. Ray hits cap at angle (will intersect with the edge of the cone)
    // 2. Ray hits cap straight on (will not intersect any edge but will hit tip)

    let origin = ray.origin();
    let direction = ray.direction();
    let height = -HALF_HEIGHT;

    let t = (height - origin.y) / direction.y;
    // Return as soon as possible to avoid extra work
    if !t_range.contains(&t) {
        return None;
    }

    let hit_point = ray.at(t);
    // Check if point is within the circle
    if (hit_point.x*hit_point.x + hit_point.z*hit_point.z) > RADIUS*RADIUS {
        return None;
    }

    // Bottom cap normal always points down
    let normal = Vec3::down();

    Some(RayIntersection {
        ray_parameter: t,
        hit_point,
        normal,
        tex_coord: None,
    })
}

impl RayHit for Cone {
    fn ray_hit(&self, ray: &Ray, init_t_range: &Range<f64>) -> Option<RayIntersection> {
        // A cone is actually two parts:
        // 2. Hollow cone body (x^2 + z^2 - (y - h/2)^2 = 0, -HALF_HEIGHT <= y <= HALF_HEIGHT)
        // 3. Bottom cap (y = -HALF_HEIGHT)
        //
        // We need to test all three in order to be fully robust. After all, we can't guarantee
        // which order the ray will hit these parts in. That means that we can't quit as soon as
        // we find a hit in any of them. Luckily, we can use t_range to optimize only returning
        // a hit if it is in the valid range.

        let mut t_range = init_t_range.clone();
        let mut found_hit = None;

        if let Some(hit) = ray_hit_body(ray, &t_range) {
            // Must find a closer hit next time to be accepted
            t_range.end = hit.ray_parameter;
            found_hit = Some(hit);
        }

        // Try bottom cap
        if let Some(hit) = ray_hit_cap(ray, &t_range) {
            // Just accept the hit if we find it, no point in updating t_range
            found_hit = Some(hit);
        }

        found_hit
    }
}
