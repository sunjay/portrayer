use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};
use crate::math::Vec3;

use super::bounding_volume::BoundingVolume;
#[cfg(not(feature = "render_bounding_volumes"))]
use super::triangle::Triangle;

/// A 3D mesh made of triangles.
#[derive(Debug)]
pub struct Mesh {
    /// Each item is a group of three vertices (their indexes) representing a triangle
    triangles: Vec<(usize, usize, usize)>,
    /// The position of each vertex
    positions: Vec<Vec3>,
    /// The bounding volume that forms the dimensions of this mesh
    bounds: Option<BoundingVolume>,
}

impl<'a> From<&'a tobj::Mesh> for Mesh {
    fn from(mesh: &'a tobj::Mesh) -> Self {
        let triangles = mesh.indices.chunks_exact(3)
            .map(|t| (t[0] as usize, t[1] as usize, t[2] as usize))
            .collect();
        let positions = mesh.positions.chunks_exact(3)
            .map(|p| Vec3 {x: p[0] as f64, y: p[1] as f64, z: p[2] as f64})
            .collect();

        Self::new(positions, triangles)
    }
}

impl Mesh {
    fn new(positions: Vec<Vec3>, triangles: Vec<(usize, usize, usize)>) -> Self {
        let bounds = BoundingVolume::new(&positions);
        Self {
            triangles,
            positions,
            bounds,
        }
    }
}

#[cfg(not(feature = "render_bounding_volumes"))]
impl RayHit for Mesh {
    fn ray_hit(&self, ray: &Ray, init_t_range: &Range<f64>) -> Option<RayIntersection> {
        // Test the bounding volume first. If it does not get hit we can save a lot of time that
        // we would have spent traversing vertices.
        if let Some(bounds) = self.bounds {
            if !bounds.check_hit(ray, init_t_range) {
                return None;
            }
        }

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
