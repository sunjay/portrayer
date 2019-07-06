use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Vec3, Uv};
use crate::bounding_box::{BoundingBox, Bounds};

use super::plane::Plane;

/// L = length/width/height of the cube
const L: f64 = 1.0;
const L2: f64 = L / 2.0;

/// An axis-aligned unit cube with center (0, 0, 0) and width/length/height 1.0
///
/// It is expected that this cube will be used via affine transformations on the node that
/// contains it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cube;

impl Cube {
    /// Returns true if the given point is anywhere within the *volume* of the cube
    pub fn contains(self, Vec3 {x, y, z}: Vec3) -> bool {
        // Need to add epsilon when doing these checks to account for floating point error. Without
        // this we get lots of "unfilled" spots ("shadow acne") all over the cube and its shadow.
        let radius = L2 + EPSILON;
        -radius <= x && x <= radius && -radius <= y && y <= radius && -radius <= z && z <= radius
    }
}

impl Bounds for Cube {
    fn bounds(&self) -> BoundingBox {
        let min = Vec3::from(-L2);
        let max = Vec3::from(L2);
        BoundingBox::new(min, max)
    }
}

impl RayHit for Cube {
    fn ray_hit(&self, ray: &Ray, init_t_range: &Range<f64>) -> Option<RayIntersection> {
        // Define the six faces of a cube and the conversion to each face's texture coordinate
        // system: (plane, axis_direction, texture_offset)
        //
        // Texture coordinate offset comes from the cubemap being a 4x3 grid:
        // Each offset can be calculated from (col * 1/4, row * 1/3) where col = 0..=2 and row = 0..=3
        // For a reference image: https://learnopengl.com/Advanced-OpenGL/Cubemaps
        static FACES: [(Plane, Uv, Uv); 6] = [
            // Right
            (Plane {point: Vec3 {x: L2, y: 0.0, z: 0.0}, normal: Vec3 {x: 1.0, y: 0.0, z: 0.0}},
                Uv {u: -1.0, v: 1.0}, Uv {u: 1.0/2.0, v: 1.0/3.0}),
            // Left
            (Plane {point: Vec3 {x: -L2, y: 0.0, z: 0.0}, normal: Vec3 {x: -1.0, y: 0.0, z: 0.0}},
                Uv {u: 1.0, v: 1.0}, Uv {u: 0.0, v: 1.0/3.0}),
            // Top
            (Plane {point: Vec3 {x: 0.0, y: L2, z: 0.0}, normal: Vec3 {x: 0.0, y: 1.0, z: 0.0}},
                Uv {u: 1.0, v: -1.0}, Uv {u: 1.0/4.0, v: 0.0}),
            // Bottom
            (Plane {point: Vec3 {x: 0.0, y: -L2, z: 0.0}, normal: Vec3 {x: 0.0, y: -1.0, z: 0.0}},
                Uv {u: 1.0, v: 1.0}, Uv {u: 1.0/4.0, v: 2.0/3.0}),
            // Near
            (Plane {point: Vec3 {x: 0.0, y: 0.0, z: L2}, normal: Vec3 {x: 0.0, y: 0.0, z: 1.0}},
                Uv {u: 1.0, v: 1.0}, Uv {u: 1.0/4.0, v: 1.0/3.0}),
            // Far
            (Plane {point: Vec3 {x: 0.0, y: 0.0, z: -L2}, normal: Vec3 {x: 0.0, y: 0.0, z: -1.0}},
                Uv {u: -1.0, v: 1.0}, Uv {u: 3.0/4.0, v: 1.0/3.0}),
        ];

        //TODO: Experiment with parallelism via rayon (might not be worth it for 6 checks)

        // Find the nearest intersection
        let mut t_range = init_t_range.clone();
        FACES.iter().fold(None, |hit, (plane, uv_axis, uv_offset)| {
            match plane.ray_hit(ray, &t_range) {
                // Need to check if the cube actually contains the hit point since each
                // plane is infinite
                Some(mut p_hit) => if self.contains(p_hit.hit_point) {
                    // Compute texture coordinate by finding 2D intersection coordinate on cube face

                    // Get the uv coordinate on the face by finding the right set of two points
                    let hit_p = p_hit.hit_point;
                    let face_uv = match plane.normal {
                        n if n.x != 0.0 => Uv {u: hit_p.z, v: hit_p.y},
                        n if n.y != 0.0 => Uv {u: hit_p.x, v: hit_p.z},
                        n if n.z != 0.0 => Uv {u: hit_p.x, v: hit_p.y},
                        _ => unreachable!(),
                    };

                    // Convert face to normalized image coordinate system with +x to the right and
                    // +y down
                    let norm_uv = Uv {
                        u: face_uv.u * uv_axis.u + L2,
                        v: L2 - face_uv.v * uv_axis.v,
                    };

                    // Convert from the coordinate system of this particular face texture to the
                    // full coordinate system of the cube map. Models how face_uv is a coordinate
                    // in one of the 6 images of the full 4x3 cube map.
                    let global_uv = norm_uv / Uv {u: 4.0, v: 3.0} + uv_offset;
                    p_hit.tex_coord = Some(global_uv);

                    // Limit the search of the next face using the current t value
                    t_range.end = p_hit.ray_parameter;
                    Some(p_hit)
                } else { hit },
                None => hit,
            }
        })
    }
}
