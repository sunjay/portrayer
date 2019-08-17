use std::sync::Arc;
use std::ops::Range;

use crate::math::Vec3;
use crate::material::Material;
use crate::bounding_box::{BoundingBox, Bounds};
use crate::ray::{RayCast, RayHit, Ray, RayIntersection};
use crate::primitive::{InfinitePlane, InfinitePlaneRight, InfinitePlaneUp, InfinitePlaneFront, PlaneSide};

use super::KDTreeNode;

/// A node and its bounding box
///
/// Cached to avoid computing the bounding box from the node over and over again.
#[derive(Debug, PartialEq)]
pub(crate) struct NodeBounds<T> {
    pub bounds: BoundingBox,
    pub node: T,
}

impl<T> Bounds for NodeBounds<T> {
    fn bounds(&self) -> BoundingBox {
        self.bounds.clone()
    }
}

impl<T: Bounds> From<T> for NodeBounds<T> {
    fn from(node: T) -> Self {
        Self {
            bounds: node.bounds(),
            node,
        }
    }
}

impl<T: RayCast> RayCast for NodeBounds<T> {
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)> {
        // In the future we could potentially use the bounding box for a sort of BVH optimization.
        // This isn't necessary right now though because Mesh already uses its own BVH and no other
        // primitive needs that extra optimization.
        self.node.ray_cast(ray, t_range)
    }
}

impl<T: RayHit> RayHit for NodeBounds<T> {
    fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
        // In the future we could potentially use the bounding box for a sort of BVH optimization.
        // This isn't necessary right now though because Mesh already uses its own BVH and no other
        // primitive needs that extra optimization.
        self.node.ray_hit(ray, t_range)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PartitionConfig {
    /// The target maximum number of nodes allowed in a leaf node. A leaf may have more or less
    /// nodes than this depending on how partitioning goes.
    pub target_max_nodes: usize,
    /// The target maximum difference between the number of items in front or behind the separating
    /// plane. Note that this includes items shared by both partitions.
    /// merit = (front - back).abs() + shared
    pub target_max_merit: isize,

    /// The maximum number of attempts to find a good separating plane. After this, we will
    /// just take whatever we get.
    pub max_tries: usize,
}

/// The axis being partitioned on
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PartitionAxis {
    X,
    Y,
    Z,
}

impl Default for PartitionAxis {
    fn default() -> Self {
        // The default axis to start partitioning on
        PartitionAxis::X
    }
}

impl From<PartitionAxis> for Vec3 {
    fn from(axis: PartitionAxis) -> Vec3 {
        match axis {
            PartitionAxis::X => Vec3::up(),
            PartitionAxis::Y => Vec3::right(),
            PartitionAxis::Z => Vec3::back_rh(),
        }
    }
}

impl PartitionAxis {
    /// Returns the next axis in the partitioning sequence
    fn next(self) -> Self {
        use PartitionAxis::*;
        match self {
            X => Y,
            Y => Z,
            Z => X,
        }
    }

    /// Creates an infinite plane (a separating plane) for the given partitioning axis
    fn sep_plane(self, point: Vec3) -> InfinitePlane {
        use PartitionAxis::*;
        match self {
            X => InfinitePlaneRight {point}.into(),
            Y => InfinitePlaneUp {point}.into(),
            Z => InfinitePlaneFront {point}.into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct KDLeaf<T> {
    /// A bounding box that encompases all of the scene nodes in this leaf node
    pub bounds: BoundingBox,
    /// The scene nodes to be tested for intersection
    ///
    /// Need to store in Arc because scene nodes can be shared between multiple tree nodes if
    /// the scene node could not be evenly split by the separating plane
    pub nodes: Vec<Arc<NodeBounds<T>>>,
}

impl<T> KDLeaf<T> {
    /// Partition the nodes in this leaf until the number of nodes is less than the given
    /// threshold or until the leaf cannot be partitioned anymore. There is no guarantee that the
    /// resulting tree will have fewer nodes in its leaves than the given threshold, but we will
    /// try our best.
    ///
    /// The provided axis vector must be a positive unit vector: (1,0,0), (0,1,0), or (0,0,1)
    ///
    /// When max_depth == 0, the remaining nodes will be returned in a single leaf node
    pub(in super) fn partitioned(
        self,
        axis: PartitionAxis,
        max_depth: usize,
        part_conf: PartitionConfig,
    ) -> KDTreeNode<T> {
        let PartitionConfig {target_max_nodes, target_max_merit, max_tries} = part_conf;
        if max_depth == 0 || self.nodes.len() <= target_max_nodes {
            return KDTreeNode::Leaf(self);
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Partition {
            Front,
            Back,
            Shared,
        }

        /// Tests which side of the separating plane a given node is on. The node may be on both
        /// sides. Returns (front, back) where each is true if the node is on that side.
        fn partition_node<T>(
            node: &Arc<NodeBounds<T>>,
            sep_plane: &InfinitePlane,
        ) -> Partition {
            use PlaneSide::*;

            let node_min = node.bounds.min();
            let node_max = node.bounds.max();

            match (sep_plane.which_side(node_min), sep_plane.which_side(node_max)) {
                // Node is entirely in front of the separating plane
                (Front, Front) => Partition::Front,
                // Node is entirely behind the separating plane
                (Back, Back) => Partition::Back,
                // Node is both in front and behind
                (Front, Back) | (Back, Front) => Partition::Shared,
            }
        }

        let KDLeaf {bounds, nodes} = self;

        // Find the center of the bounding box along the given axis
        let axis_vec = Vec3::from(axis);
        let min_axis = axis_vec * bounds.min();
        let max_axis = axis_vec * bounds.max();
        // The plane is infinite, so it doesn't actually matter where this point is
        // (e.g. it does not need to depend on the previous split if any)
        let mut sep_plane = axis.sep_plane(min_axis + (max_axis - min_axis) / 2.0);

        // This variable represents the valid range that the partitioning plane can exist in
        // Once we find that we need to move the plane forwards or backwards, the valid range
        // becomes either the area in front of the plane or the area behind.
        let mut plane_range = (min_axis, max_axis);

        //TODO: This is a simpler (and less efficient) algorithm than the one in the paper. We
        // partition the same list of nodes over and over again with different plane choices. They
        // only partition the nodes on the side of the plane that need to be repartitioned. We can
        // experiment with the more complex (but potentially faster) method later on.
        //TODO: Consider tracking the "best" separating plane based on the "merit" merit and then
        // returning that instead if we hit MAX_TRIES
        for _ in 0..max_tries {
            // Time-space/allocation trade-off: not going to partition the nodes until we've
            // actually decided on a good partition. This avoids allocating over and over again
            // for partitions we aren't even going to keep.

            // The number of nodes in front of the separating plane
            let mut front = 0isize;
            // The number of nodes behind the separating plane
            let mut back = 0isize;
            // The number of nodes that are partially in front and partially behind the plane
            let mut shared = 0isize;
            for node in &nodes {
                match partition_node(node, &sep_plane) {
                    Partition::Front => front += 1,
                    Partition::Back => back += 1,
                    Partition::Shared => shared += 1,
                }
            }

            // Determine how good the partition is
            let merit = (front - back).abs() + shared;
            if merit <= target_max_merit {
                break;
            }

            // Pick a new separating plane (similar to a simple binary search)

            // Note that this code assumes that `axis` is a positive single-axis unit vector. Thus:
            // * Every node behind the plane is between min_axis and sep_plane.point.
            // * Every node in front of the plane is between sep_plane.point and max_axis.

            // The separating plane is currently in this range:
            let sep_plane_point = sep_plane.point();
            let (plane_min, plane_max) = plane_range;
            if front > back {

                // plane must be in the forward half of its range
                plane_range = (sep_plane_point, plane_max);
                // Move plane forward
                *sep_plane.point_mut() += (plane_max - sep_plane_point) / 2.0;

            } else {
                // plane must be in the back half of its range
                plane_range = (plane_min, sep_plane_point);
                // Move plane backward
                *sep_plane.point_mut() = plane_min + (sep_plane_point - plane_min) / 2.0;
            }
        }

        // Create the actual partition based on the chosen plane
        let mut front_nodes = Vec::new();
        let mut back_nodes = Vec::new();
        for node in nodes {
            match partition_node(&node, &sep_plane) {
                Partition::Front => front_nodes.push(node),
                Partition::Back => back_nodes.push(node),
                Partition::Shared => {
                    front_nodes.push(node.clone());
                    back_nodes.push(node);
                },
            }
        }

        let next = axis.next();
        KDTreeNode::Split {
            sep_plane,
            // Copy the bounds from the original leaf since it already encompases all the nodes
            bounds,
            front_nodes: Box::new(KDLeaf {
                bounds: front_nodes.bounds(),
                nodes: front_nodes,
            }.partitioned(next, max_depth - 1, part_conf)),
            back_nodes: Box::new(KDLeaf {
                bounds: back_nodes.bounds(),
                nodes: back_nodes,
            }.partitioned(next, max_depth - 1, part_conf)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::math::{Mat4, Vec3};
    use crate::flat_scene::FlatSceneNode;
    use crate::material::Material;
    use crate::scene::Geometry;
    use crate::primitive::{Plane};

    use super::super::KDTreeNode;

    #[test]
    fn single_axis_center_partition() {
        // 5 objects:  A   B    S   C  D  E
        //        x = -8  -5    0   3  5  8
        //               back       front
        //
        // With target_max_nodes = 3, we should get two leaf nodes separated by the plane S
        let part_conf = PartitionConfig {
            target_max_nodes: 3,
            target_max_merit: 3,
            max_tries: 10,
        };

        let mat = Arc::new(Material::default());

        let make_node_bounds = |x| {
            let node = FlatSceneNode::new(Geometry::new(Plane, mat.clone()),
                Mat4::rotation_z(90.0f64.to_radians()).translated_3d((x, 0.0, 0.0)));
            Arc::new(NodeBounds {bounds: node.bounds(), node})
        };

        let node_a = make_node_bounds(-8.0);
        let node_b = make_node_bounds(-5.0);
        let node_c = make_node_bounds(3.0);
        let node_d = make_node_bounds(5.0);
        let node_e = make_node_bounds(8.0);

        let nodes = vec![node_a.clone(), node_b.clone(), node_c.clone(), node_d.clone(), node_e.clone()];
        let nodes_bounds = nodes.bounds();
        let leaf = KDLeaf {
            bounds: nodes_bounds.clone(),
            nodes,
        };

        let root = leaf.partitioned(PartitionAxis::default(), 5, part_conf);

        let back_nodes = vec![node_a, node_b];
        let front_nodes = vec![node_c, node_d, node_e];
        let expected_root = KDTreeNode::Split {
            sep_plane: InfinitePlaneRight {point: Vec3::zero()}.into(),
            bounds: nodes_bounds,
            front_nodes: Box::new(KDTreeNode::Leaf(KDLeaf {
                bounds: front_nodes.bounds(),
                nodes: front_nodes,
            })),
            back_nodes: Box::new(KDTreeNode::Leaf(KDLeaf {
                bounds: back_nodes.bounds(),
                nodes: back_nodes,
            })),
        };

        assert_eq!(expected_root, root);
    }

    #[test]
    fn single_axis_uneven_partition() {
        // 5 objects:  A        B   C  D  E
        //        x = -8        0   3  5  8
        //               back       front
        //                           ^------ expected separating plane, x = 4.0
        //
        // With target_max_nodes = 3, we should get one split followed by two leaf nodes.
        // Note that the separating plane is not the center anymore. It will take more than one
        // iteration to find it.
        let part_conf = PartitionConfig {
            target_max_nodes: 3,
            target_max_merit: 2,
            max_tries: 10,
        };

        let mat = Arc::new(Material::default());

        let make_node_bounds = |x| {
            let node = FlatSceneNode::new(Geometry::new(Plane, mat.clone()),
                Mat4::rotation_z(90.0f64.to_radians()).translated_3d((x, 0.0, 0.0)));
            Arc::new(NodeBounds {bounds: node.bounds(), node})
        };

        let node_a = make_node_bounds(-8.0);
        let node_b = make_node_bounds(0.0);
        let node_c = make_node_bounds(3.0);
        let node_d = make_node_bounds(5.0);
        let node_e = make_node_bounds(8.0);

        let nodes = vec![node_a.clone(), node_b.clone(), node_c.clone(), node_d.clone(), node_e.clone()];
        let nodes_bounds = nodes.bounds();
        let leaf = KDLeaf {
            bounds: nodes_bounds.clone(),
            nodes,
        };

        let root = leaf.partitioned(PartitionAxis::default(), 5, part_conf);

        let back_nodes = vec![node_a, node_b, node_c];
        let front_nodes = vec![node_d, node_e];
        let expected_root = KDTreeNode::Split {
            sep_plane: InfinitePlaneRight {
                point: Vec3 {x: 4.0, y: 0.0, z: 0.0},
            }.into(),
            bounds: nodes_bounds,
            front_nodes: Box::new(KDTreeNode::Leaf(KDLeaf {
                bounds: front_nodes.bounds(),
                nodes: front_nodes,
            })),
            back_nodes: Box::new(KDTreeNode::Leaf(KDLeaf {
                bounds: back_nodes.bounds(),
                nodes: back_nodes,
            })),
        };

        assert_eq!(expected_root, root);
    }
}
