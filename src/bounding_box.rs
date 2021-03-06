use std::sync::Arc;
use std::ops::{Mul, Range};

use crate::math::{EPSILON, INFINITY, Vec3, Vec3Ext, Mat4};
use crate::ray::{Ray, RayHit};
use crate::primitive::Cube;

#[cfg(feature = "render_bounding_volumes")]
use crate::ray::RayIntersection;

pub trait Bounds {
    /// Returns a bounding box that fully encapsulates this object
    fn bounds(&self) -> BoundingBox;
}

impl<T: Bounds> Bounds for Arc<T> {
    fn bounds(&self) -> BoundingBox {
        (&*self as &T).bounds()
    }
}

/// Finds the maximum bounding box around a list of objects
impl<T: Bounds> Bounds for Vec<T> {
    fn bounds(&self) -> BoundingBox {
        if self.is_empty() {
            return BoundingBox::new(Vec3::zero(), Vec3::zero());
        }

        let first = self[0].bounds();
        let (min, max) = self.iter().skip(1).fold((first.min(), first.max()), |(min, max), item| {
            let item_bounds = item.bounds();
            (Vec3::partial_min(min, item_bounds.min()), Vec3::partial_max(max, item_bounds.max()))
        });

        BoundingBox::new(min, max)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    /// The corner of the bounding box with all the lowest x,y,z values
    min: Vec3,
    /// The corner of the bounding box with all the highest x,y,z values
    max: Vec3,
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
            min,
            max,
            #[cfg(feature = "render_bounding_volumes")]
            trans,
            invtrans,
            #[cfg(feature = "render_bounding_volumes")]
            normal_trans,
        }
    }

    /// Returns the corner of the bounding box with the minimum x,y,z values
    pub fn min(&self) -> Vec3 {
        self.min
    }

    /// Returns the corner of the bounding box with the maximum x,y,z values
    pub fn max(&self) -> Vec3 {
        self.max
    }

    /// Returns the maximum distance between any two points within the bounding box
    pub fn extent(&self) -> f64 {
        //HACK: Using magnitude_squared instead of magnitude or else the k-d tree will miss points.
        //TODO: Find a way to do this without the hack.
        (self.max - self.min).magnitude_squared()
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

/// Allows a bounding box to be transformed by a transformation matrix
///
/// Syntax: transform * bounding_box
///     This is similar to the syntax: transform * point
impl Mul<BoundingBox> for Mat4 {
    type Output = BoundingBox;

    fn mul(self, rhs: BoundingBox) -> Self::Output {
        // Turns out that transforming an Axis-Aligned Bounding Box and then generating a new AABB
        // is non-trivial. You can't just transform the min/max points because when the other
        // corners of the box rotate too they may end up being the new min/max. That's why we need
        // to actually simulate all 8 points of the AABB being transformed.

        let mut min = Vec3::from(INFINITY);
        let mut max = Vec3::from(-INFINITY);

        // Generate all points of the cube, transform them and find the min/max
        for &x in &[rhs.min.x, rhs.max.x] {
            for &y in &[rhs.min.y, rhs.max.y] {
                for &z in &[rhs.min.z, rhs.max.z] {
                    let cube_vert = Vec3 {x, y, z}.transformed_point(self);
                    min = Vec3::partial_min(min, cube_vert);
                    max = Vec3::partial_max(max, cube_vert);
                }
            }
        }

        BoundingBox::new(min, max)
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::math::Mat4;

    #[test]
    fn rotated_plane_bounds_90() {
        let bounds = BoundingBox::new(
            Vec3 {x: -0.5, y: 0.0, z: -0.5},
            Vec3 {x: 0.5, y: 0.0, z: 0.5},
        );

        let trans = Mat4::rotation_x(90.0f64.to_radians());
        let rotated_bounds = trans * bounds;
        assert_eq!(rotated_bounds.min().map(|x| (x * 10.0).round() / 10.0), Vec3 {x: -0.5, y: -0.5, z: 0.0});
        assert_eq!(rotated_bounds.max().map(|x| (x * 10.0).round() / 10.0), Vec3 {x: 0.5, y: 0.5, z: 0.0});
    }

    #[test]
    fn rotated_cube_bounds_60() {
        let bounds = BoundingBox::new(
            Vec3 {x: -0.5, y: -0.5, z: -0.5},
            Vec3 {x: 0.5, y: 0.5, z: 0.5},
        );

        let trans = Mat4::scaling_3d((8.0, 0.25, 5.0)).rotated_x(60.0f64.to_radians());
        let rotated_bounds = trans * bounds;
        assert_eq!(rotated_bounds.min().map(|x| (x * 1000.0).round() / 1000.0), Vec3 {x: -4.0, y: -2.228, z: -1.358});
        assert_eq!(rotated_bounds.max().map(|x| (x * 1000.0).round() / 1000.0), Vec3 {x: 4.0, y: 2.228, z: 1.358});
    }
}
