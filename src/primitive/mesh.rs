use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::{EPSILON, Vec3, Mat4};
#[cfg(feature = "render_bounding_volumes")]
use crate::math::Vec3Ext;

use super::Cube;
#[cfg(not(feature = "render_bounding_volumes"))]
use super::triangle::Triangle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shading {
    /// Flat shading - the vertex normals will be ignored and a single normal will be generated
    /// based on the edges of the mesh face
    Flat,
    /// Phong (smooth) shading - the vertex normals will be interpolated
    Smooth,
}

/// A 3D mesh made of triangles.
#[derive(Debug)]
pub struct Mesh {
    /// Each item is a group of three vertices (their indexes) representing a triangle
    triangles: Vec<(usize, usize, usize)>,
    /// The position of each vertex
    positions: Vec<Vec3>,
    /// Vertex normals. Only used if shading == Smooth
    normals: Vec<Vec3>,
    /// The mode to use when computing the normal of each face
    shading: Shading,
    /// The type of shading to use when computing the normal
    /// Transforms the bounding volume (a cube) to wrap around the the mesh
    #[cfg(feature = "render_bounding_volumes")]
    bounds_trans: Mat4,
    /// Transforms the ray into the original coordinate system of cube for hit calculations
    inv_bounds_trans: Mat4,
    /// Transforms the normal of the hit point back into the mesh coordinate system
    #[cfg(feature = "render_bounding_volumes")]
    bounds_normal_trans: Mat4,
}

impl Mesh {
    /// Creates a new mesh with the given vertices and triangles.
    pub fn new(mesh: &tobj::Mesh, shading: Shading) -> Self {
        let triangles = mesh.indices.chunks_exact(3)
            .map(|t| (t[0] as usize, t[1] as usize, t[2] as usize))
            .collect();
        let positions: Vec<_> = mesh.positions.chunks_exact(3)
            .map(|p| Vec3 {x: p[0] as f64, y: p[1] as f64, z: p[2] as f64})
            .collect();
        let normals: Vec<_> = mesh.normals.chunks_exact(3)
            .map(|p| Vec3 {x: p[0] as f64, y: p[1] as f64, z: p[2] as f64})
            .collect();

        if shading == Shading::Smooth {
            assert_eq!(positions.len(), normals.len(),
                "Meshes must have a vertex normal for each vertex if they are to be used with smooth shading");
        }

        // Compute bounding cube
        //TODO: Experiment with parallelism via rayon for computing bounds (benchmark)
        assert!(!positions.is_empty(), "Meshes must have at least one vertex");
        let p0 = positions[0];
        let (min, max) = positions.iter().skip(1).fold((p0, p0), |(min, max), &vert| {
            (Vec3::partial_min(min, vert), Vec3::partial_max(max, vert))
        });

        let bounds_size = max - min;
        // Special-case: planes and other 2D objects
        // Need a non-zero scale because otherwise the matrix is not invertable (and we'll get NaN)
        let bounds_size = Vec3::partial_max(bounds_size, EPSILON.into());

        // Find the center of the bounding volume
        let center = (min + max) / 2.0;

        let bounds_trans = Mat4::scaling_3d(bounds_size).translated_3d(center);
        let inv_bounds_trans = bounds_trans.inverted();
        #[cfg(feature = "render_bounding_volumes")]
        let bounds_normal_trans = inv_bounds_trans.transposed();

        Self {
            triangles,
            positions,
            normals,
            shading,
            #[cfg(feature = "render_bounding_volumes")]
            bounds_trans,
            inv_bounds_trans,
            #[cfg(feature = "render_bounding_volumes")]
            bounds_normal_trans,
        }
    }
}

#[cfg(not(feature = "render_bounding_volumes"))]
impl RayHit for Mesh {
    fn ray_hit(&self, ray: &Ray, init_t_range: &Range<f64>) -> Option<RayIntersection> {
        // Test the bounding volume first. If it does not get hit we can save a lot of time that
        // we would have spent traversing vertices.

        // Take the ray from its current coordinate system and put it into the local coordinate
        // system of the bounding volume
        let local_ray = ray.transformed(self.inv_bounds_trans);
        if Cube.ray_hit(&local_ray, init_t_range).is_none() {
            return None;
        }

        //TODO: Parallelism via rayon

        let mut t_range = init_t_range.clone();
        self.triangles.iter().fold(None, |hit, &(a, b, c)| {
            use Shading::*;
            let tri = Triangle {
                a: self.positions[a],
                b: self.positions[b],
                c: self.positions[c],
                normals: match self.shading {
                    Flat => None,
                    Smooth => Some((self.normals[a], self.normals[b], self.normals[c])),
                },
            };

            match tri.ray_hit(ray, &t_range) {
                Some(hit) => {
                    t_range.end = hit.ray_parameter;
                    Some(hit)
                },
                None => hit,
            }
        })
    }
}

#[cfg(feature = "render_bounding_volumes")]
impl RayHit for Mesh {
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
