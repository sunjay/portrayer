use std::sync::Arc;
use std::ops::Range;

use crate::scene::Scene;
use crate::material::Material;
use crate::flat_scene::{FlatScene, FlatSceneNode};
use crate::bounding_box::{BoundingBox, Bounds};
use crate::ray::{RayCast, RayHit, Ray, RayIntersection};
use crate::primitive::{Plane, PlaneSide};

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
        back_nodes: Box<KDTreeNode>,
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
            Leaf {nodes, ..} => nodes.ray_cast(ray, t_range),
            Split {sep_plane, bounds, front_nodes, back_nodes} => {
                // A value of t large enough that the point on the ray for this t would be well
                // beyond the extent of the scene
                let t_max = bounds.extent();
                // Must still be a value in the valid range
                let t_max = if t_range.contains(&t_max) { t_max } else { t_range.end };
                // The two "end points" of the ray make a "ray segment"
                let ray_start = ray.at(t_range.start);
                let ray_end = ray.at(t_max);

                // Search through child nodes, filtering and ordering by which side(s) of the
                // separating plane is hit by the ray segment
                use PlaneSide::*;
                match (sep_plane.which_side(ray_start), sep_plane.which_side(ray_end)) {
                    // Ray segment lies entirely on front side of the separating plane
                    (Front, Front) => front_nodes.ray_cast(ray, t_range),

                    // Ray segment lies entirely on back side of the separating plane
                    (Back, Back) => back_nodes.ray_cast(ray, t_range),

                    // Ray segment goes through the front nodes, then the back nodes
                    (Front, Back) => {
                        // Must ensure that any found intersection is actually on the checked side
                        // of the plane or else we can get incorrect results
                        let plane_hit = sep_plane.ray_hit(ray, t_range)
                            .expect("bug: ray should definitely hit infinite plane");
                        let plane_t = plane_hit.ray_parameter;

                        // Only going to continue with this range if it hits
                        let mut front_t_range = Range {start: t_range.start, end: plane_t};
                        match front_nodes.ray_cast(ray, &mut front_t_range) {
                            Some(hit_mat) => {
                                *t_range = front_t_range;
                                Some(hit_mat)
                            },
                            None => {
                                // Only going to continue with this range if it hits
                                let mut back_t_range = Range {start: plane_t, end: t_range.end};
                                match back_nodes.ray_cast(ray, &mut back_t_range) {
                                    Some(hit_mat) => {
                                        *t_range = back_t_range;
                                        Some(hit_mat)
                                    },
                                    None => None,
                                }
                            },
                        }
                    },
                    // Ray segment goes through the back nodes, then the front nodes
                    (Back, Front) => {
                        // Must ensure that any found intersection is actually on the checked side
                        // of the plane or else we can get incorrect results
                        // Need to flip the plane since the ray is facing the back of the plane
                        let plane_hit = sep_plane.flipped().ray_hit(ray, t_range)
                            .expect("bug: ray should definitely hit infinite plane");
                        let plane_t = plane_hit.ray_parameter;

                        // Only going to continue with this range if it hits
                        let mut back_t_range = Range {start: t_range.start, end: plane_t};
                        match back_nodes.ray_cast(ray, &mut back_t_range) {
                            Some(hit_mat) => {
                                *t_range = back_t_range;
                                Some(hit_mat)
                            },
                            None => {
                                // Only going to continue with this range if it hits
                                let mut front_t_range = Range {start: plane_t, end: t_range.end};
                                match front_nodes.ray_cast(ray, &mut front_t_range) {
                                    Some(hit_mat) => {
                                        *t_range = front_t_range;
                                        Some(hit_mat)
                                    },
                                    None => None,
                                }
                            },
                        }
                    },
                }
            },
        }
    }
}
