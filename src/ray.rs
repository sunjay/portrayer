use std::f64::{EPSILON, INFINITY};
use std::ops::Range;

use crate::math::{Vec3, Vec4, Mat4, Rgb};
use crate::scene::{Scene, SceneNode};
use crate::material::Material;

/// Represents the result of a ray intersection and stores information about it
#[derive(Debug)]
pub struct RayIntersection<'a> {
    /// The smallest positive value of t for which the given ray intersects the target. Note that
    /// the smaller the t value, the closer the intersection is to the origin of the ray.
    ray_parameter: f64,

    /// The point of intersection
    hit_point: Vec3,

    /// The normal at the point of intersection.
    /// IMPORTANT: This is NOT guaranteed to be a unit vector for the sake of efficiency and
    /// floating point correctness. (Normalizing too many times accrues too much floating point
    /// error.) Make sure you normalize when it matters.
    normal: Vec3,

    /// The material of the geometry that was intersected
    material: &'a Material,
}

pub trait RayHit {
    /// Returns a value if the given ray has hit this object and the parameter is in the given range
    fn ray_hit(&self, ray: &Ray, range: &Range<f64>) -> Option<RayIntersection>;
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
        assert!(direction.is_normalized(), "bug: ray direction must be normalized");
        Self {origin, direction}
    }

    /// Computes the position in this ray at the given ray parameter value
    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Transforms the ray by the given matrix and returns a new copy with the transformed result
    pub fn transformed(&self, trans: Mat4) -> Self {
        Self {
            origin: Vec3::from(trans * Vec4::from_point(self.origin)),
            direction: Vec3::from(trans * Vec4::from_direction(self.direction)),
        }
    }

    /// Cast the ray in its configured direction and return the nearest geometry that it intersects
    pub fn cast(&self, node: &SceneNode, t_range: &mut Range<f64>) -> Option<RayIntersection> {
        unimplemented!()
    }

    /// Compute the color of the nearest object to the casted ray. Returns the given background
    /// color if no object is hit by this ray.
    pub fn color(&self, scene: &Scene, background: Rgb) -> Rgb {
        let mut t_range = Range {start: EPSILON, end: INFINITY};
        let hit = self.cast(&scene.root, &mut t_range);

        match hit {
            Some(hit) => {
                hit.material.hit_color(scene, background, self.direction, hit.hit_point, hit.normal)
            },
            None => background,
        }
    }
}
