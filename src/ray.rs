use std::ops::Range;
use std::sync::Arc;

use crate::math::{EPSILON, INFINITY, Vec3, Vec3Ext, Mat4, Mat3, Rgb, Uv};
use crate::scene::Scene;
use crate::material::Material;

/// Represents the result of a ray intersection and stores information about it
#[derive(Debug, Clone, PartialEq)]
pub struct RayIntersection {
    /// The smallest positive value of t for which the given ray intersects the target. Note that
    /// the smaller the t value, the closer the intersection is to the origin of the ray.
    pub ray_parameter: f64,

    /// The point of intersection
    pub hit_point: Vec3,

    /// The normal at the point of intersection.
    /// IMPORTANT: This is NOT guaranteed to be a unit vector for the sake of efficiency and
    /// floating point correctness. (Normalizing too many times accrues too much floating point
    /// error.) Make sure you normalize when it matters.
    pub normal: Vec3,

    /// The texture coordinate of the hit point (if any)
    ///
    /// Set to None if the surface does not support texture mapping.
    pub tex_coord: Option<Uv>,

    /// The matrix to compute the normal from a normal in a normal map
    ///
    /// The normal applied to this matrix will have a right-handed, y-up coordinate system where
    /// a normal (nx,ny,nz) perpendicular to the surface is (0.0,1.0,0.0)
    ///
    /// Set to None if the surface does not support normal mapping.
    pub normal_map_transform: Option<Mat3>,
}

/// Abstracts the ray hitting a single primitive
pub trait RayHit {
    /// Returns a value if the given ray has hit this object and the parameter is in the given range
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection>;
}

impl<T: RayHit> RayHit for Arc<T> {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        (&*self as &T).ray_hit(ray, t_range)
    }
}

impl<T: RayHit> RayHit for [T] {
    fn ray_hit(&self, ray: &Ray, init_t_range: &Range<f64>) -> Option<RayIntersection> {
        let mut t_range = init_t_range.clone();
        self.iter().fold(None, |hit, item| match item.ray_hit(ray, &t_range) {
            Some(hit) => {
                // Only allow further intersections if they are closer to the ray origin
                // than this one
                t_range.end = hit.ray_parameter;
                Some(hit)
            },
            None => hit,
        })
    }
}

/// Abstracts the ray casting through the entire hierarchical structure of a scene
pub trait RayCast {
    /// Cast the ray and find the nearest geometry that it intersects.
    ///
    /// The given t_range value should have its `end` field updated to the nearest t value found.
    ///
    /// Returned value contains information about what was hit and its material.
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)>;
}

impl<T: RayCast> RayCast for Arc<T> {
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)> {
        (&*self as &T).ray_cast(ray, t_range)
    }
}

impl<T: RayCast> RayCast for Vec<T> {
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)> {
        (&*self as &[T]).ray_cast(ray, t_range)
    }
}

impl<T: RayCast> RayCast for [T] {
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)> {
        self.iter().fold(None, |hit_mat, item| match item.ray_cast(ray, t_range) {
            Some((hit, mat)) => {
                // Only allow further intersections if they are closer to the ray origin
                // than this one
                t_range.end = hit.ray_parameter;
                Some((hit, mat))
            },
            None => hit_mat,
        })
    }
}

#[derive(Debug)]
pub struct Ray {
    /// The initial point of this ray (ray parameter t = 0.0)
    origin: Vec3,
    /// The direction of this ray (MUST be normalized)
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {origin, direction}
    }

    /// Returns the origin position of this ray
    pub fn origin(&self) -> Vec3 {
        self.origin
    }

    /// Returns the direction of this ray
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Computes the position in this ray at the given ray parameter value
    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Transforms the ray by the given matrix and returns a new copy with the transformed result
    pub fn transformed(&self, trans: Mat4) -> Self {
        Self {
            origin: self.origin.transformed_point(trans),
            direction: self.direction.transformed_direction(trans),
        }
    }

    /// Compute the color of the nearest object to the casted ray. Returns the given background
    /// color if no object is hit by this ray.
    pub fn color<R: RayCast>(&self, scene: &Scene<R>, background: Rgb, recursion_depth: u32) -> Rgb {
        let mut t_range = Range {start: EPSILON, end: INFINITY};
        let hit = scene.root.ray_cast(self, &mut t_range);

        match hit {
            Some((hit, mat)) => mat.hit_color(scene, background, self.direction, hit.hit_point,
                hit.normal, hit.tex_coord, hit.normal_map_transform, recursion_depth),
            None => background,
        }
    }
}
