use std::sync::Arc;
use std::ops::Range;

use crate::scene::Scene;
use crate::material::Material;
use crate::flat_scene::{FlatScene, FlatSceneNode};
use crate::bounding_box::{BoundingBox, Bounds};
use crate::ray::{RayCast, Ray, RayIntersection};
use crate::primitive::Plane;

/// A scene organized as a KDTree for fast intersections
pub type KDTreeScene = Scene<KDTreeNode>;

/// Builds a k-d tree from a flattened scene
impl From<FlatScene> for KDTreeScene {
    fn from(flat_scene: FlatScene) -> Self {
        let FlatScene {root: flat_nodes, lights, ambient} = flat_scene;

        // Turn the entire scene into a single, unpartitioned leaf node
        let nodes: Vec<_> = flat_nodes.into_iter()
            .map(|node| NodeBounds::from(node).into())
            .collect();
        let root = KDTreeNode::Leaf {bounds: nodes.bounds(), nodes};

        Self {root, lights, ambient}
    }
}

/// A node and its bounding box
///
/// Cached to avoid computing the bounding box from the node over and over again.
#[derive(Debug)]
pub struct NodeBounds {
    bounds: BoundingBox,
    node: FlatSceneNode,
}

impl Bounds for Arc<NodeBounds> {
    fn bounds(&self) -> BoundingBox {
        self.bounds.clone()
    }
}

impl From<FlatSceneNode> for NodeBounds {
    fn from(node: FlatSceneNode) -> Self {
        Self {
            bounds: node.bounds(),
            node,
        }
    }
}

impl RayCast for Arc<NodeBounds> {
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)> {
        // In the future we could potentially use the bounding box for a sort of BVH optimization.
        // This isn't necessary right now though because Mesh already uses its own BVH and no other
        // primitive needs that extra optimization.
        self.node.ray_cast(ray, t_range)
    }
}

#[derive(Debug)]
pub enum KDTreeNode {
    Split {
        /// The separating plane that divides the children
        sep_plane: Plane,
        /// A bounding box that encompases all of the nodes in this node
        bounds: BoundingBox,
        /// The nodes in front of the separating plane (in the direction of the plane normal)
        front_nodes: Box<KDTreeNode>,
        /// The nodes behind the separating plane (in the direction opposite to the plane normal)
        behind_nodes: Box<KDTreeNode>,
    },
    Leaf {
        /// A bounding box that encompases all of the scene nodes in this leaf node
        bounds: BoundingBox,
        /// The scene nodes to be tested for intersection
        ///
        /// Need to store in Arc because scene nodes can be shared between multiple tree nodes if
        /// the scene node could not be evenly split by the separating plane
        nodes: Vec<Arc<NodeBounds>>,
    },
}

impl RayCast for KDTreeNode {
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)> {
        use KDTreeNode::*;
        match self {
            Split {..} => unimplemented!(),
            Leaf {nodes, ..} => nodes.ray_cast(ray, t_range),
        }
    }
}
