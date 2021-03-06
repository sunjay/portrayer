use std::env;

use crate::scene::Scene;
use crate::math::Vec3;
use crate::bounding_box::Bounds;
use crate::flat_scene::{FlatScene, FlatSceneNode};

use super::{KDTreeNode, KDLeaf, NodeBounds, PartitionConfig};

/// The maximum depth of any k-d tree
///
/// Can be set via the KD_DEPTH environment variable
const MAX_TREE_DEPTH: usize = 10;

/// A scene organized as a KDTree for fast intersections
pub(crate) type KDTreeScene = Scene<KDTreeNode<FlatSceneNode>>;

/// Builds a k-d tree from a flattened scene
impl From<FlatScene> for KDTreeScene {
    fn from(flat_scene: FlatScene) -> Self {
        let FlatScene {root: flat_nodes, lights, ambient} = flat_scene;

        // Turn the entire scene into a single, unpartitioned leaf node
        let nodes: Vec<_> = flat_nodes.into_iter()
            .map(|node| NodeBounds::from(node).into())
            .collect();

        let leaf = KDLeaf {bounds: nodes.bounds(), nodes};
        let part_conf = PartitionConfig {
            target_max_nodes: 3,
            target_max_merit: 3,
            max_tries: 10,
        };

        // Allow overriding the max tree depth for bigger scenes
        let max_tree_depth = env::var("KD_DEPTH").ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(MAX_TREE_DEPTH);

        let root = leaf.partitioned(Vec3::unit_x(), max_tree_depth, part_conf);

        Self {root, lights, ambient}
    }
}
