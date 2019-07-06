use std::sync::Arc;
use std::ops::Range;

use crate::math::{Mat4, Vec3Ext};
use crate::scene::Scene;
use crate::material::Material;
use crate::flat_scene::{FlatScene, FlatSceneNode};
use crate::bounding_box::{BoundingBox, Bounds};
use crate::ray::{RayCast, RayHit, Ray, RayIntersection};

/// A scene organized as a KDTree for fast intersections
pub type KDTreeScene = Scene<KDTreeNode>;

/// Builds a k-d tree from a flattened scene
impl From<FlatScene> for KDTreeScene {
    fn from(flat_scene: FlatScene) -> Self {
        let FlatScene {root: nodes, lights, ambient} = flat_scene;

        // Start with a single, unpartitioned node
        let root = KDTreeNode::flat_internal_node(nodes);

        Self {root, lights, ambient}
    }
}

#[derive(Debug)]
pub enum KDTreeNode {
    Internal {
        /// A bounding box that encompases all of the children of this node
        bounds: BoundingBox,
        children: Vec<KDTreeNode>,
    },
    Leaf {
        /// A bounding box that only encompases this node
        ///
        /// Cached to avoid computing the bounding box from the node over and over again.
        bounds: BoundingBox,
        node: FlatSceneNode,
    },
}

impl KDTreeNode {
    /// Creates a new, unpartitioned node where every child is from the given list
    fn flat_internal_node(nodes: Vec<FlatSceneNode>) -> Self {
        let children: Vec<_> = nodes.into_iter().map(|node| KDTreeNode::Leaf {
            bounds: node.bounds(),
            node,
        }).collect();

        KDTreeNode::Internal {
            bounds: children.bounds(),
            children,
        }
    }
}

impl Bounds for KDTreeNode {
    fn bounds(&self) -> BoundingBox {
        use KDTreeNode::*;
        match self {
            Internal {bounds, ..} | Leaf {bounds, ..} => bounds.clone(),
        }
    }
}

impl RayCast for KDTreeNode {
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)> {
        unimplemented!()
    }
}
