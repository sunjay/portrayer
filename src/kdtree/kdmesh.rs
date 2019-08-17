use std::env;
use std::sync::Arc;
use std::ops::Range;

use crate::bounding_box::{BoundingBox, Bounds};
use crate::primitive::{MeshData, Shading, Triangle};
use crate::ray::{RayHit, Ray, RayIntersection};

use super::{KDTreeNode, KDLeaf, PartitionConfig, PartitionAxis, NodeBounds};

/// The maximum depth of any k-d tree
///
/// Can be set via the KD_MESH_DEPTH environment variable
const MAX_TREE_DEPTH: usize = 10;

/// A Mesh backed by a k-d tree to store the triangles
#[derive(Debug, Clone, PartialEq)]
pub struct KDMesh {
    // Storing the triangles in an Arc to make this cheap to clone without duplicating the tree.
    // This is very important in case the node containing this primitive is instanced and then
    // flattened. It's the same reason why Mesh stores Arc<MeshData>.
    triangles: Arc<KDTreeNode<Triangle>>,
}

impl Bounds for KDMesh {
    fn bounds(&self) -> BoundingBox {
        self.triangles.bounds().clone()
    }
}

impl KDMesh {
    /// Creates a new mesh from the given mesh data and with the given shading
    ///
    /// Note that this does not store the given mesh data. Instead it copies the data into the
    /// nodes of a k-d tree.
    pub fn new(data: &MeshData, shading: Shading) -> Self {
        // Turn all of the mesh triangles into a single, unpartitioned leaf node
        let nodes: Vec<_> = data.triangles(shading)
            .map(|node| NodeBounds::from(node).into())
            .collect();

        let leaf = KDLeaf {bounds: nodes.bounds(), nodes};
        let part_conf = PartitionConfig {
            target_max_nodes: 3,
            target_max_merit: 3,
            max_tries: 10,
        };

        // Allow overriding the max tree depth for bigger scenes
        let max_tree_depth = env::var("KD_MESH_DEPTH").ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(MAX_TREE_DEPTH);

        let root = leaf.partitioned(PartitionAxis::default(), max_tree_depth, part_conf);

        Self {triangles: Arc::new(root)}
    }
}

#[cfg(not(feature = "render_bounding_volumes"))]
impl RayHit for KDMesh {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // Test the bounding volume first. If it does not get hit we can save a lot of time that
        // we would have spent traversing the mesh triangles. This is important for the KDMesh but
        // not the KDTreeScene because it's far less likely that a ray would miss the entire scene
        // than it is that a ray would miss a given mesh.
        if self.triangles.bounds().test_hit(ray, t_range).is_none() {
            return None;
        }

        self.triangles.ray_hit(ray, t_range)
    }
}

#[cfg(feature = "render_bounding_volumes")]
impl RayHit for KDMesh {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // Pretend that this mesh is the bounding volume and test that instead
        self.triangles.bounds().ray_hit(ray, t_range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;

    use rayon::prelude::*;

    use crate::math::{Vec3, Rgb, Radians};
    use crate::primitive::{Mesh, MeshData, Shading};
    use crate::material::Material;
    use crate::camera::{Camera, CameraSettings};
    use crate::scene::{HierScene, SceneNode, Geometry};
    use crate::light::Light;

    #[test]
    fn mesh_equivalence() -> Result<(), Box<dyn Error>> {
        // Test that all the same points are hit for both meshes and k-d meshes

        let mat_castle_walls = Arc::new(Material {
            diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
            specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
            shininess: 25.0,
            ..Material::default()
        });

        let model = Arc::new(MeshData::load_obj("assets/castle.obj")?);
        let scene_kd_mesh = HierScene {
            root: SceneNode::from(Geometry::new(KDMesh::new(&model, Shading::Flat), mat_castle_walls.clone()))
                .scaled(1.4)
                .translated((0.0, 0.0, -229.0))
                .into(),

            lights: vec![
                Light {
                    position: Vec3 {x: 50.0, y: 110.0, z: -120.0},
                    color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                    ..Light::default()
                }
            ],
            ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
        };
        let scene_mesh = HierScene {
            root: SceneNode::from(Geometry::new(Mesh::new(model, Shading::Flat), mat_castle_walls.clone()))
                .scaled(1.4)
                .translated((0.0, 0.0, -229.0))
                .into(),

            lights: vec![
                Light {
                    position: Vec3 {x: 50.0, y: 110.0, z: -120.0},
                    color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                    ..Light::default()
                }
            ],

            ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
        };

        let cam = CameraSettings {
            eye: (0.0, 120.0, 240.0).into(),
            center: (0.0, 100.0, -24.0).into(),
            up: Vec3::up(),
            fovy: Radians::from_degrees(25.0),
        };
        let width = 533.0;
        let height = 300.0;
        let camera = Camera::new(cam, (width, height));

        // Ray cast against the front of the monkey's face
        let n = 100000;
        (0..n).into_par_iter().zip((0..n).into_par_iter()).panic_fuse().for_each(|(i, j)| {
            let x = width * i as f64 / n as f64;
            let y = height * j as f64 / n as f64;

            let ray = camera.ray_at((x, y));

            assert_eq!(ray.color(&scene_mesh, Rgb::black(), 0), ray.color(&scene_kd_mesh, Rgb::black(), 0),
                "pixels at (x={}, y={}) were not the same", x, y);
        });

        Ok(())
    }
}
