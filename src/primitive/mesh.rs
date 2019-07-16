use std::ops::Range;
use std::sync::Arc;
use std::path::Path;

use crate::math::{Vec3, Uv};
use crate::ray::{Ray, RayHit, RayIntersection};
use crate::bounding_box::{BoundingBox, Bounds};

use super::Triangle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shading {
    /// Flat shading - the vertex normals will be ignored and a single normal will be generated
    /// based on the edges of the mesh face
    Flat,
    /// Phong (smooth) shading - the vertex normals will be interpolated
    Smooth,
}

/// The 3D data of a mesh, can be shared between multiple Meshes
#[derive(Debug, PartialEq)]
pub struct MeshData {
    /// Each item is a group of three vertices (their indexes) representing a triangle
    triangles: Vec<(usize, usize, usize)>,
    /// The position of each vertex
    positions: Vec<Vec3>,
    /// Vertex normals. Only used if shading == Smooth
    normals: Vec<Vec3>,
    /// Texture coordinates for each vertex. If provided, must have enough for each vertex.
    tex_coords: Vec<Uv>,
    /// A bounding box that encompases all vertices of this mesh. Used to avoid having to test all
    /// triangles if we can already trivially know that there is no intersection.
    bounds: BoundingBox,
}

impl<'a> From<&'a tobj::Mesh> for MeshData {
    fn from(mesh: &'a tobj::Mesh) -> Self {
        let triangles = mesh.indices.chunks_exact(3)
            .map(|t| (t[0] as usize, t[1] as usize, t[2] as usize))
            .collect();
        let positions = mesh.positions.chunks_exact(3)
            .map(|p| Vec3 {x: p[0] as f64, y: p[1] as f64, z: p[2] as f64})
            .collect();
        let normals = mesh.normals.chunks_exact(3)
            .map(|p| Vec3 {x: p[0] as f64, y: p[1] as f64, z: p[2] as f64})
            .collect();
        let tex_coords = mesh.texcoords.chunks_exact(2)
            .map(|uv| Uv {u: uv[0] as f64, v: uv[1] as f64})
            .collect();

        MeshData::new(positions, triangles, normals, tex_coords)
    }
}

impl MeshData {
    /// Loads a *single* mesh (the first mesh) from an OBJ file
    pub fn load_obj<P: AsRef<Path>>(path: P) -> Result<Self, tobj::LoadError> {
        let path = path.as_ref();
        let (models, _) = tobj::load_obj(path)?;
        Ok(MeshData::from(&models[0].mesh))
    }

    pub fn new(
        positions: Vec<Vec3>,
        triangles: Vec<(usize, usize, usize)>,
        normals: Vec<Vec3>,
        tex_coords: Vec<Uv>,
    ) -> Self {
        // Compute bounding cube
        //TODO: Experiment with parallelism via rayon for computing bounds (benchmark)
        assert!(!positions.is_empty(), "Meshes must have at least one vertex");
        let p0 = positions[0];
        let (min, max) = positions.iter().skip(1).fold((p0, p0), |(min, max), &vert| {
            (Vec3::partial_min(min, vert), Vec3::partial_max(max, vert))
        });

        if !tex_coords.is_empty() && tex_coords.len() != positions.len() {
            panic!("If meshes have texture coordinates, they must have enough for all vertices");
        }

        Self {
            triangles,
            positions,
            normals,
            tex_coords,
            bounds: BoundingBox::new(min, max),
        }
    }

    /// Iterate through all the triangles represented by this data. The shading parametering
    /// affects whther the yielded triangles are provided normals from the mesh or not.
    ///
    /// Note that if shading == Smooth you are guaranteeing that there is at least one normal per
    /// vertex.
    pub fn triangles(&self, shading: Shading) -> impl Iterator<Item=Triangle> + '_ {
        self.triangles.iter().map(move |&(a, b, c)| {
            use Shading::*;
            Triangle {
                a: self.positions[a],
                b: self.positions[b],
                c: self.positions[c],
                normals: match shading {
                    Flat => None,
                    Smooth => Some((self.normals[a], self.normals[b], self.normals[c])),
                },
                tex_coords: if self.tex_coords.is_empty() {
                    None
                } else {
                    Some((self.tex_coords[a], self.tex_coords[b], self.tex_coords[c]))
                }
            }
        })
    }
}

/// A 3D mesh made of triangles.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    data: Arc<MeshData>,
    /// The mode to use when computing the normal of each face
    shading: Shading,
}

impl Bounds for Mesh {
    fn bounds(&self) -> BoundingBox {
        self.data.bounds.clone()
    }
}

impl Mesh {
    /// Creates a new mesh from the given mesh data and with the given shading
    pub fn new(data: Arc<MeshData>, shading: Shading) -> Self {
        if shading == Shading::Smooth {
            assert_eq!(data.positions.len(), data.normals.len(),
                "Meshes must have a vertex normal for each vertex if they are to be used with smooth shading");
        }

        Self {
            data,
            shading,
        }
    }
}

#[cfg(not(feature = "render_bounding_volumes"))]
impl RayHit for Mesh {
    fn ray_hit(&self, ray: &Ray, init_t_range: &Range<f64>) -> Option<RayIntersection> {
        let data = &self.data;

        // Test the bounding volume first. If it does not get hit we can save a lot of time that
        // we would have spent traversing the mesh triangles.
        if data.bounds.test_hit(ray, init_t_range).is_none() {
            return None;
        }

        let mut t_range = init_t_range.clone();
        //TODO: Parallelism via rayon
        data.triangles(self.shading).fold(None, |hit, tri| {
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
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // Pretend that this mesh is the bounding volume and test that instead
        self.data.bounds.ray_hit(ray, t_range)
    }
}
