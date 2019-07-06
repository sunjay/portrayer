use std::ops::Range;

use crate::math::{EPSILON, Vec3, Mat4};
use crate::ray::{Ray, RayHit};
use crate::primitive::Cube;

#[cfg(feature = "render_bounding_volumes")]
use crate::math::Vec3Ext;
#[cfg(feature = "render_bounding_volumes")]
use crate::ray::RayIntersection;

#[derive(Debug)]
pub struct BoundingBox {
    /// Transforms the unit cube to full dimensions of the bounding box
    #[cfg(feature = "render_bounding_volumes")]
    trans: Mat4,
    /// Transforms the ray into the coordinate system of the unit cube for hit calculations
    invtrans: Mat4,
    /// Transforms the normal of the hit point back into the bounding box coordinate system
    #[cfg(feature = "render_bounding_volumes")]
    normal_trans: Mat4,
}

impl BoundingBox {
    /// Create a new bounding box from the given minimum and maximum points
    pub fn new(min: Vec3, max: Vec3) -> Self {
        assert!(min.partial_cmple(&max).reduce_and(), "bounding box min must be less than max");

        let size = max - min;
        // Special-case: planes and other 2D objects
        // Need a non-zero scale because otherwise the matrix is not invertable (and we'll get NaN)
        let size = Vec3::partial_max(size, EPSILON.into());

        // Find the center of the bounding volume
        let center = (min + max) / 2.0;

        let trans = Mat4::scaling_3d(size).translated_3d(center);
        let invtrans = trans.inverted();
        #[cfg(feature = "render_bounding_volumes")]
        let normal_trans = invtrans.transposed();

        Self {
            #[cfg(feature = "render_bounding_volumes")]
            trans,
            invtrans,
            #[cfg(feature = "render_bounding_volumes")]
            normal_trans,
        }
    }

    /// Returns the ray parameter value for which this bounding box will be hit by the given ray
    ///
    /// If the ray at t_range.start is inside the bounding box, t_range.start will be returned.
    pub fn test_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<f64> {
        // Take the ray from its current coordinate system and put it into the local coordinate
        // system of the bounding box
        let local_ray = ray.transformed(self.invtrans);
        // If the minimum ray parameter value results in a point inside the cube, the ray
        // unconditionally intersects with this bounding box even if none of the edges will be hit
        // by it. (None of the edges will be hit because the normals face outwards)
        if Cube.contains(local_ray.at(t_range.start)) {
            return Some(t_range.start);
        }

        Cube.ray_hit(&local_ray, t_range).map(|hit| hit.ray_parameter)
    }
}

#[cfg(feature = "render_bounding_volumes")]
impl RayHit for BoundingBox {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // Take the ray from its current coordinate system and put it into the local coordinate
        // system
        let local_ray = ray.transformed(self.invtrans);
        Cube.ray_hit(&local_ray, t_range).map(|mut hit| {
            // Need to transform hit_point and normal back so they render properly
            hit.hit_point = hit.hit_point.transformed_point(self.trans);
            hit.normal = hit.normal.transformed_direction(self.normal_trans);
            hit
        })
    }
}
