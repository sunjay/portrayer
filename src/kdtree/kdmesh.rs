use std::sync::Arc;
use std::ops::Range;

use crate::math::Vec3;
use crate::bounding_box::{BoundingBox, Bounds};
use crate::primitive::{MeshData, Shading, Triangle};
use crate::ray::{RayHit, Ray, RayIntersection};

use super::{KDTreeNode, KDLeaf, PartitionConfig, NodeBounds};

/// The maximum depth of any k-d tree
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
        let root = leaf.partitioned(Vec3::unit_x(), MAX_TREE_DEPTH, part_conf);

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
