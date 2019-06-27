use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Mat4, Vec3};
#[cfg(feature = "render_bounding_volumes")]
use crate::math::Vec3Ext;

use super::Cube;

/// For applying the bounding volume hierarchy optimiziation
#[derive(Debug)]
pub struct BoundingVolume {
    /// Transforms the bounding volume (a cube) to wrap around the the mesh
    #[cfg(feature = "render_bounding_volumes")]
    bounds_trans: Mat4,
    /// Transforms the ray into the original coordinate system of cube for hit calculations
    inv_bounds_trans: Mat4,
    /// Transforms the normal of the hit point back into the mesh coordinate system
    #[cfg(feature = "render_bounding_volumes")]
    bounds_normal_trans: Mat4,
}

impl BoundingVolume {
    /// Attempts to create a bounding volume from the given positions
    ///
    /// Returns None if the bounding volume would have zero volume (e.g. flat surface)
    pub fn new(positions: &[Vec3]) -> Option<Self> {
        assert!(!positions.is_empty(), "Meshes must have at least one vertex");

        // Compute bounding cube
        let p0 = positions[0];
        let (min, max, total) = positions.iter().fold((p0, p0, p0), |(min, max, total), &vert| {
            (Vec3::partial_min(min, vert), Vec3::partial_max(max, vert), total + vert)
        });

        // The true center of the geometry is the average of all the points
        let center = total / positions.len() as f64;

        let bounds_size = max - min;
        // Special-case: planes and other 2D objects
        // If the scale is zero, the matrix is not invertable (and we'll get NaN)
        if bounds_size.iter().any(|b| b.abs() < EPSILON) {
            return None;
        }

        let bounds_trans = Mat4::scaling_3d(bounds_size).translated_3d(center);
        let inv_bounds_trans = bounds_trans.inverted();
        #[cfg(feature = "render_bounding_volumes")]
        let bounds_normal_trans = inv_bounds_trans.transposed();

        Some(Self {
            #[cfg(feature = "render_bounding_volumes")]
            bounds_trans,
            inv_bounds_trans,
            #[cfg(feature = "render_bounding_volumes")]
            bounds_normal_trans,
        })
    }

    /// Check if the given ray hit this bounding volume in the given range
    pub fn check_hit(&self, ray: &Ray, t_range: &Range<f64>) -> bool {
        // Take the ray from its current coordinate system and put it into the local coordinate
        // system of the bounding volume
        let local_ray = ray.transformed(self.inv_bounds_trans);
        Cube.ray_hit(&local_ray, t_range).is_some()
    }
}

#[cfg(feature = "render_bounding_volumes")]
impl RayHit for BoundingVolume {
    fn ray_hit(&self, ray: &Ray, init_t_range: &Range<f64>) -> Option<RayIntersection> {
        // Pretend that this mesh is the bounding volume and test that instead

        // Take the ray from its current coordinate system and put it into the local coordinate
        // system of the bounding volume
        let local_ray = ray.transformed(self.inv_bounds_trans);
        Cube.ray_hit(&local_ray, init_t_range).map(|mut hit| {
            // Need to transform hit_point and normal back so they render properly
            hit.hit_point = hit.hit_point.transformed_point(self.bounds_trans);
            hit.normal = hit.normal.transformed_direction(self.bounds_normal_trans);
            hit
        })
    }
}
