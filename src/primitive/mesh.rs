use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::Vec3;

use super::triangle::Triangle;

/// A 3D mesh made of triangles.
#[derive(Debug)]
pub struct Mesh {
    /// Each item is a group of three vertices (their indexes) representing a triangle
    triangles: Vec<(usize, usize, usize)>,
    /// The position of each vertex
    positions: Vec<Vec3>,
}

impl<'a> From<&'a tobj::Mesh> for Mesh {
    fn from(mesh: &'a tobj::Mesh) -> Self {
        Self {
            triangles: mesh.indices.chunks_exact(3)
                .map(|t| (t[0] as usize, t[1] as usize, t[2] as usize))
                .collect(),
            positions: mesh.positions.chunks_exact(3)
                .map(|p| Vec3 {x: p[0] as f64, y: p[1] as f64, z: p[2] as f64})
                .collect(),
        }
    }
}

impl RayHit for Mesh {
    fn ray_hit(&self, ray: &Ray, init_t_range: &Range<f64>) -> Option<RayIntersection> {
        //TODO: Use BVH to speed this up

        //TODO: Parallelism via rayon

        let mut t_range = init_t_range.clone();
        self.triangles.iter().fold(None, |hit, &(a, b, c)| {
            let tri = Triangle {
                a: self.positions[a],
                b: self.positions[b],
                c: self.positions[c],
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
