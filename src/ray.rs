use std::ops::Range;
use std::sync::Arc;

use crate::math::{EPSILON, INFINITY, Vec3, Vec3Ext, Mat4, Rgb, Uv};
use crate::scene::{Scene, SceneNode, Geometry};
use crate::material::Material;

/// Represents the result of a ray intersection and stores information about it
#[derive(Debug)]
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
    pub tex_coord: Option<Uv>,
}

pub trait RayHit {
    /// Returns a value if the given ray has hit this object and the parameter is in the given range
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection>;
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

    /// Cast the ray in its configured direction and return the nearest geometry that it intersects
    ///
    /// Returned value contains information about what was hit and its material.
    pub fn cast<'a>(
        &self,
        node: &'a SceneNode,
        t_range: &mut Range<f64>,
    ) -> Option<(RayIntersection, Arc<Material>)> {
        // Take the ray from its current coordinate system and put it into the local coordinate
        // system of the current node
        let local_ray = self.transformed(node.inverse_trans());

        // These will be used to transform the hit point and normal back into the
        // previous coordinate system
        let trans = node.trans();
        let normal_trans = node.normal_trans();

        // The resulting hit and material (initially None)
        let mut hit_mat = None;

        // Check if the ray intersects this node's geometry (if any)
        if let Some(Geometry {primitive, material}) = node.geometry() {
            if let Some(mut hit) = primitive.ray_hit(&local_ray, t_range) {
                hit.hit_point = hit.hit_point.transformed_point(trans);
                hit.normal = hit.normal.transformed_direction(normal_trans);

                // Only allow further intersections if they are closer to the ray origin
                // than this one
                t_range.end = hit.ray_parameter;

                hit_mat = Some((hit, material.clone()));
            }
        }

        // Recurse into children and attempt to find a closer match
        for child in node.children() {
            if let Some((mut child_hit, child_mat)) = local_ray.cast(child, t_range) {
                child_hit.hit_point = child_hit.hit_point.transformed_point(trans);
                child_hit.normal = child_hit.normal.transformed_direction(normal_trans);

                // No need to set t_range.end since it is set in the recursive base case of cast()

                hit_mat = Some((child_hit, child_mat));
            }
        }

        hit_mat
    }

    /// Compute the color of the nearest object to the casted ray. Returns the given background
    /// color if no object is hit by this ray.
    pub fn color(&self, scene: &Scene, background: Rgb, recursion_depth: u32) -> Rgb {
        let mut t_range = Range {start: EPSILON, end: INFINITY};
        let hit = self.cast(&scene.root, &mut t_range);

        match hit {
            Some((hit, mat)) => mat.hit_color(scene, background, self.direction, hit.hit_point,
                hit.normal, hit.tex_coord, recursion_depth),
            None => background,
        }
    }
}
