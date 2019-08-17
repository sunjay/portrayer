use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Vec3, Mat3, Uv};
use crate::bounding_box::{BoundingBox, Bounds};

use super::{
    InfinitePlaneUp,
    InfinitePlaneDown,
    InfinitePlaneRight,
    InfinitePlaneLeft,
    InfinitePlaneFront,
    InfinitePlaneBack,
};

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
        // Find the nearest intersection
        let mut t_range = init_t_range.clone();
        // The nearest plane and hit (if any)
        let mut nearest_plane_hit = None;

        // Using a macro to avoid duplicating the code for each type of plane
        macro_rules! intersect_face {
            ($plane:expr, $axis_direction:expr, $texture_offset:expr) => {
                let plane = $plane;

                if let Some(hit) = plane.ray_hit(ray, &t_range) {
                    // Need to check if the cube actually contains the hit point since each
                    // plane is infinite
                    if self.contains(hit.hit_point) {
                        // Limit the search of the next face using the current t value so we only
                        // get another hit if it is closer to the ray origin
                        t_range.end = hit.ray_parameter;
                        nearest_plane_hit = Some((plane.normal(), $axis_direction, $texture_offset, hit))
                    }
                }
            };
        }

        // Intersect the six faces of the cube and the conversion to each face's texture coordinate
        // system: (plane, axis_direction, texture_offset)
        //
        // Texture coordinate offset comes from the cubemap being a 4x3 grid:
        // Each offset can be calculated from (col * 1/4, row * 1/3) where col = 0..=2 and row = 0..=3
        // For a reference image: https://learnopengl.com/Advanced-OpenGL/Cubemaps

        // Right
        intersect_face!(InfinitePlaneRight {point: Vec3 {x: L2, y: 0.0, z: 0.0}},
            Uv {u: -1.0, v: 1.0}, Uv {u: 1.0/2.0, v: 1.0/3.0});
        // Left
        intersect_face!(InfinitePlaneLeft {point: Vec3 {x: -L2, y: 0.0, z: 0.0}},
            Uv {u: 1.0, v: 1.0}, Uv {u: 0.0, v: 1.0/3.0});
        // Top
        intersect_face!(InfinitePlaneUp {point: Vec3 {x: 0.0, y: L2, z: 0.0}},
            Uv {u: 1.0, v: -1.0}, Uv {u: 1.0/4.0, v: 0.0});
        // Bottom
        intersect_face!(InfinitePlaneDown {point: Vec3 {x: 0.0, y: -L2, z: 0.0}},
            Uv {u: 1.0, v: 1.0}, Uv {u: 1.0/4.0, v: 2.0/3.0});
        // Near
        intersect_face!(InfinitePlaneFront {point: Vec3 {x: 0.0, y: 0.0, z: L2}},
            Uv {u: 1.0, v: 1.0}, Uv {u: 1.0/4.0, v: 1.0/3.0});
        // Far
        intersect_face!(InfinitePlaneBack {point: Vec3 {x: 0.0, y: 0.0, z: -L2}},
            Uv {u: -1.0, v: 1.0}, Uv {u: 3.0/4.0, v: 1.0/3.0});

        // Additional hit properties are computed once at the end to avoid wasted computations
        if let Some((normal, uv_axis, uv_offset, mut hit)) = nearest_plane_hit {
            // Compute texture coordinate by finding 2D intersection coordinate on cube face

            // Get the uv coordinate on the face by finding the right set of two points
            let hit_p = hit.hit_point;
            let face_uv = match normal {
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
            hit.tex_coord = Some(global_uv);

            // To find the normal map transform, we need a basis for each face that aligns the face
            // normal with the right-handed y-axis. That means that for the majority of the faces,
            // we need a horizontal tangent on the xz-plane and a vertical tangent perpendicular to
            // both the normal and the horizontal tangent. For the special case of the top and
            // bottom faces, we can use the standard right-handed basis rotated up or down

            // To find the horizontal tangent, we can take advantage of the top face being "above"
            // every other face.
            let to_top = (Vec3 {x: 0.0, y: L, z: 0.0} - hit.hit_point).normalized();
            let normal_map_transform = if to_top.x.abs() < EPSILON && to_top.z.abs() < EPSILON {
                // Special case: top or bottom face, return standard basis aligned with normal
                Mat3::from_col_arrays([
                    Vec3::right().into_array(),
                    normal.into_array(),
                    if normal.y > 0.0 { Vec3::back_rh() } else { Vec3::forward_rh() }.into_array(),
                ])
            } else {
                let horizontal_tangent = to_top.cross(normal);
                let vertical_tangent = normal.cross(horizontal_tangent);
                Mat3::from_col_arrays([
                    horizontal_tangent.into_array(),
                    normal.into_array(),
                    vertical_tangent.into_array(),
                ])
            };
            hit.normal_map_transform = Some(normal_map_transform);

            Some(hit)
        } else {
            None
        }
    }
}
