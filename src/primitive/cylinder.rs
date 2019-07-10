use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{Vec3, Quadratic};
use crate::bounding_box::{BoundingBox, Bounds};

/// The radius of the cylinder
const RADIUS: f64 = 0.5;
const HEIGHT: f64 = 1.0;
const HALF_HEIGHT: f64 = HEIGHT / 2.0;

/// A cylinder with center (0, 0, 0), diameter = 1.0, and height = 1.0
///
/// It is expected that this cylinder will be used via affine transformations on the node that
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

/// Attempt to intersect with the side of the cylinder
fn ray_hit_body(ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
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
    let t = equation.solve().find(|sol| t_range.contains(sol))?;
    // Stop any operations as early as possible if we're not in the valid range
    if !t_range.contains(&t) {
        return None;
    }

    let hit_point = ray.at(t);
    // Test if we went beyond the caps
    if hit_point.y > HALF_HEIGHT || hit_point.y < -HALF_HEIGHT {
        return None;
    }

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

/// Attempt to intersect with the cap of the cylinder
fn ray_hit_cap(height: f64, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
    // An easy way to test the cap is to treat it like a plane where the intersection point has
    // to satisfy: x^2 + z^2 <= r^2
    //
    // We don't even have to use the entire general plane intersection routine. We can exploit the
    // fact that the plane is axis-aligned and do something much cheaper. Since the plane is
    // axis-aligned to the y-axis, all intersection points must have the same y-value (`height`).
    // That means that we need only solve the y-component of the ray equation: r.y = p.y + t*d.y
    // with r.y = height. This will give us a t value. Then we can just test the x and z values
    // to see if we satisfy x^2 + z^2 <= r^2 and we're good to go.

    // Note that this solution is robust against every case:
    // 1. Ray hits cap at angle (will intersect with the edge of the cylinder)
    // 2. Ray hits cap straight on (will not intersect any edge)
    //
    // Some solutions online do not account for the second case and try to (incorrectly) derive
    // this intersection from the t values of the cylinder body ray hits.

    let origin = ray.origin();
    let direction = ray.direction();

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

    // Normal can be found from the normalized height since height is positive for the top cap
    // and negative for the bottom cap
    let normal = Vec3 {x: 0.0, y: height / height.abs(), z: 0.0};

    Some(RayIntersection {
        ray_parameter: t,
        hit_point,
        normal,
        tex_coord: None,
    })
}

impl RayHit for Cylinder {
    fn ray_hit(&self, ray: &Ray, init_t_range: &Range<f64>) -> Option<RayIntersection> {
        // A cylinder is actually three parts:
        // 1. Top cap (y = HALF_HEIGHT)
        // 2. Hollow cylinder body (x^2 + z^2 = r^2, -HALF_HEIGHT <= y <= HALF_HEIGHT)
        // 3. Bottom cap (y = -HALF_HEIGHT)
        //
        // We need to test all three in order to be fully robust. After all, we can't guarantee
        // which order the ray will hit these parts in. That means that we can't quit as soon as
        // we find a hit in any of them. Luckily, we can use t_range to optimize only returning
        // a hit if it is in the valid range.

        let mut t_range = init_t_range.clone();
        let mut found_hit = None;

        // Try the body first since it has the greater surface area
        if let Some(hit) = ray_hit_body(ray, &t_range) {
            // Must find a closer hit next time to be accepted
            t_range.end = hit.ray_parameter;
            found_hit = Some(hit);
        }

        // Try each cap
        if let Some(hit) = ray_hit_cap(HALF_HEIGHT, ray, &t_range) {
            // Must find a closer hit next time to be accepted
            t_range.end = hit.ray_parameter;
            found_hit = Some(hit);
        }
        if let Some(hit) = ray_hit_cap(-HALF_HEIGHT, ray, &t_range) {
            // Must find a closer hit next time to be accepted
            t_range.end = hit.ray_parameter;
            found_hit = Some(hit);
        }

        found_hit
    }
}
