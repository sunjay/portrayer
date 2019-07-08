//! This implementation is based on this technical report:
//!
//! > Donald S. Fussell and K. R. Subramanian. Fast Ray Tracing Using K-d Trees. Tech. rep.
//! > Austin, TX, USA: University of Texas at Austin, 1988.

use std::sync::Arc;
use std::ops::Range;

use crate::math::{EPSILON, Vec3};
use crate::scene::Scene;
use crate::material::Material;
use crate::flat_scene::{FlatScene, FlatSceneNode};
use crate::bounding_box::{BoundingBox, Bounds};
use crate::ray::{RayCast, Ray, RayIntersection};
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

        let leaf = KDLeaf {bounds: nodes.bounds(), nodes};
        // Arbitrary target number of leaves in each leaf. A leaf may end up with more or less than
        // this depending on how things go during partitioning.
        let part_conf = PartitionConfig {
            target_max_nodes: 3,
            target_max_merit: 3,
            max_tries: 10,
        };
        let root = leaf.partitioned(Vec3::unit_x(), part_conf);

        Self {root, lights, ambient}
    }
}

/// A node and its bounding box
///
/// Cached to avoid computing the bounding box from the node over and over again.
#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct KDLeaf {
    /// A bounding box that encompases all of the scene nodes in this leaf node
    bounds: BoundingBox,
    /// The scene nodes to be tested for intersection
    ///
    /// Need to store in Arc because scene nodes can be shared between multiple tree nodes if
    /// the scene node could not be evenly split by the separating plane
    nodes: Vec<Arc<NodeBounds>>,
}

#[derive(Debug, Clone, Copy)]
struct PartitionConfig {
    /// The target maximum number of nodes allowed in a leaf node. A leaf may have more or less
    /// nodes than this depending on how partitioning goes.
    target_max_nodes: usize,
    /// The target maximum difference between the number of items in front or behind the separating
    /// plane. Note that this includes items shared by both partitions.
    /// merit = (front - back).abs() + shared
    target_max_merit: isize,

    /// The maximum number of attempts to find a good separating plane. After this, we will
    /// just take whatever we get.
    max_tries: usize,
}

impl KDLeaf {
    /// Partition the nodes in this leaf until the number of nodes is less than the given
    /// threshold or until the leaf cannot be partitioned anymore. There is no guarantee that the
    /// resulting tree will have fewer nodes in its leaves than the given threshold, but we will
    /// try our best.
    ///
    /// The provided axis vector must be a positive unit vector: (1,0,0), (0,1,0), or (0,0,1)
    fn partitioned(self, axis: Vec3, part_conf: PartitionConfig) -> KDTreeNode {
        let PartitionConfig {target_max_nodes, target_max_merit, max_tries} = part_conf;
        if self.nodes.len() <= target_max_nodes {
            return KDTreeNode::Leaf(self);
        }

        /// Produces the next axis to partition by "rotating"/"shifting" all elements down:
        /// (1,0,0) -> (0,1,0) -> (0,0,1)
        fn next_axis(mut axis: Vec3) -> Vec3 {
            let temp = axis.z;
            axis.z = axis.y;
            axis.y = axis.x;
            axis.x = temp;
            axis
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum Partition {
            Front,
            Back,
            Shared,
        }

        /// Tests which side of the separating plane a given node is on. The node may be on both
        /// sides. Returns (front, back) where each is true if the node is on that side.
        fn partition_node(
            node: &Arc<NodeBounds>,
            sep_plane: &Plane,
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
        let min_axis = axis * bounds.min();
        let max_axis = axis * bounds.max();
        // The plane is infinite, so it doesn't actually matter where this point is
        // (e.g. it does not need to depend on the previous split if any)
        let mut sep_plane = Plane {
            normal: axis,
            point: min_axis + (max_axis - min_axis) / 2.0,
        };

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
            let (plane_min, plane_max) = plane_range;
            if front > back {
                // Plane must be in the forward half of its range
                plane_range = (sep_plane.point, plane_max);
                // Move plane forward
                sep_plane.point = sep_plane.point + (plane_max - sep_plane.point) / 2.0;

            } else {
                // Plane must be in the back half of its range
                plane_range = (plane_min, sep_plane.point);
                // Move plane backward
                sep_plane.point = plane_min + (sep_plane.point - plane_min) / 2.0;
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

        let next = next_axis(axis);
        KDTreeNode::Split {
            sep_plane,
            // Copy the bounds from the original leaf since it already encompases all the nodes
            bounds,
            front_nodes: Box::new(KDLeaf {
                bounds: front_nodes.bounds(),
                nodes: front_nodes,
            }.partitioned(next, part_conf)),
            back_nodes: Box::new(KDLeaf {
                bounds: back_nodes.bounds(),
                nodes: back_nodes,
            }.partitioned(next, part_conf)),
        }
    }
}

#[derive(Debug, PartialEq)]
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
    Leaf(KDLeaf),
}

impl RayCast for KDTreeNode {
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)> {
        // To find the t value of the plane intersection, we exploit the fact that the plane is
        // axis-aligned and thus we already know one of the components of the hit_point that would
        // be returned from any other call to ray_hit.
        //
        // The ray_hit implementation of Plane suffers from some numerical issues when the ray is
        // right on the plane itself. This isn't typical for most scenes, but it can easily happen
        // in the k-d tree when the separating plane cross the eye point.
        //
        // This function takes advantage of the fact that the ray is defined as r(t) = p + t*d and
        // that this equation can produce the same value of t regardless of which of the following
        // alternatives we use: r_x = p_x + t*d_x   or   r_y = p_y + t*d_y   or   r_z = p_z + t*d_z
        // We can thus use the fact that (r_x, r_y, r_z) = sep_plane.point to find a value for t
        // without going through the full hit calculation. We only use the component of the plane
        // that corresponds to the non-zero component of the normal since we know that the
        // intersection point we would get from the full ray_hit must have that value for that
        // component in the hit_point it would have returned.
        fn ray_hit_axis_aligned_plane(
            sep_plane: &Plane,
            ray_start: Vec3,
            ray_end: Vec3,
            t_min: f64,
            t_range: &Range<f64>,
        ) -> Option<f64> {
            // Multiplying by the normal will set two components to zero and sum() will
            // let us fish out the non-zero value.
            let plane_value = (sep_plane.normal * sep_plane.point).sum();
            let ray_origin = (sep_plane.normal * ray_start).sum();
            let ray_direction = (sep_plane.normal * (ray_end - ray_start).normalized()).sum();

            // Need to add t_min because we used ray_start as the ray origin above
            let t = t_min + (plane_value - ray_origin) / ray_direction;

            // Must always ensure that we respect t_range or things can go *very* wrong
            if t_range.contains(&t) {
                Some(t)
            } else {
                None
            }
        }

        use KDTreeNode::*;
        match self {
            Leaf(KDLeaf {nodes, ..}) => nodes.ray_cast(ray, t_range),
            Split {sep_plane, bounds, front_nodes, back_nodes} => {
                // A value of t large enough that the point on the ray for this t would be well
                // beyond the extent of the scene. Need to add to t_range.start because otherwise
                // the bounds extent may not be enough.
                let t_max = t_range.start + bounds.extent();
                // Must still be a value in the valid range
                // Need to subtract EPSILON since range is exclusive
                let t_max = if t_range.contains(&t_max) { t_max } else { t_range.end - EPSILON };
                // The two "end points" of the ray make a "ray segment"
                // Need to add EPSILON because range is exclusive
                let t_min = t_range.start + EPSILON;
                let ray_start = ray.at(t_min);
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
                        // of the plane or else we can get incorrect results. We can do this by
                        // limiting the t_range based on the value of t for which the ray
                        // intersects the plane.

                        let plane_t = ray_hit_axis_aligned_plane(sep_plane, ray_start, ray_end, t_min, t_range)
                            .expect("bug: ray should definitely hit infinite plane");

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
                        // of the plane or else we can get incorrect results. We can do this by
                        // limiting the t_range based on the value of t for which the ray
                        // intersects the plane.

                        let plane_t = ray_hit_axis_aligned_plane(sep_plane, ray_start, ray_end, t_min, t_range)
                            .expect("bug: ray should definitely hit infinite plane");

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

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::{assert_eq, assert_ne};

    use crate::math::{EPSILON, INFINITY, Rgb, Mat4, Vec3};
    use crate::material::Material;
    use crate::scene::Geometry;
    use crate::primitive::{FinitePlane};

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
            let node = FlatSceneNode::new(Geometry::new(FinitePlane, mat.clone()),
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

        let root = leaf.partitioned(Vec3::unit_x(), part_conf);

        let back_nodes = vec![node_a, node_b];
        let front_nodes = vec![node_c, node_d, node_e];
        let expected_root = KDTreeNode::Split {
            sep_plane: Plane {normal: Vec3::unit_x(), point: Vec3::zero()},
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
            let node = FlatSceneNode::new(Geometry::new(FinitePlane, mat.clone()),
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

        let root = leaf.partitioned(Vec3::unit_x(), part_conf);

        let back_nodes = vec![node_a, node_b, node_c];
        let front_nodes = vec![node_d, node_e];
        let expected_root = KDTreeNode::Split {
            sep_plane: Plane {
                normal: Vec3::unit_x(),
                point: Vec3 {x: 4.0, y: 0.0, z: 0.0},
            },
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
    fn ray_cast_edge_case() {
        // Suppose you have the following case:
        //            S B   C
        //            ! |  /
        // R o--------!-|-/----------->
        //            !  /
        //      N <---! /
        //            !/
        //   front    /      back
        //           /!
        //          / !
        //         /  !
        // Here, the ray R is going straight towards the -z direction.
        // S is the separating plane with normal N. B and C are both polygons.
        // C is on both sides of the separating plane. B is only behind the separating plane.
        //
        // Since the ray origin is in front of the separating plane, we will traverse that side
        // first. Upon checking C, we will find an intersection. If we return that intersection,
        // the result is incorrect since B is actually in front of C. We have to check that the t
        // value returned actually indicates that the intersection is in front of S.
        //
        // If all goes well, we should return B, not C.

        let mat_b = Arc::new(Material {
            diffuse: Rgb::red(),
            ..Material::default()
        });

        let trans_b = Mat4::scaling_3d(2.0)
            .rotated_x(90f64.to_radians())
            .translated_3d((0.0, 1.2, -0.4));
        let node_b = FlatSceneNode::new(Geometry::new(FinitePlane, mat_b.clone()), trans_b);
        let b_node_bounds = Arc::new(NodeBounds {
            bounds: node_b.bounds(),
            node: node_b,
        });

        let mat_c = Arc::new(Material {
            diffuse: Rgb::blue(),
            ..Material::default()
        });
        assert_ne!(mat_b, mat_c);

        let trans_c = Mat4::scaling_3d(2.0)
            .rotated_x(50f64.to_radians())
            .translated_3d((0.0, 0.0, -0.3));
        let node_c = FlatSceneNode::new(Geometry::new(FinitePlane, mat_c.clone()), trans_c);
        let c_node_bounds = Arc::new(NodeBounds {
            bounds: node_c.bounds(),
            node: node_c,
        });

        let root = KDTreeNode::Split {
            sep_plane: Plane {normal: Vec3::unit_z(), point: Vec3::zero()},
            bounds: vec![b_node_bounds.clone(), c_node_bounds.clone()].bounds(),
            front_nodes: Box::new(KDTreeNode::Leaf(KDLeaf {
                // leaf bounds do not matter currently
                bounds: BoundingBox::new(Vec3::zero(), Vec3::zero()),
                nodes: vec![c_node_bounds.clone()],
            })),
            back_nodes: Box::new(KDTreeNode::Leaf(KDLeaf {
                // leaf bounds do not matter currently
                bounds: BoundingBox::new(Vec3::zero(), Vec3::zero()),
                // Force tree to check C again by putting it first
                nodes: vec![c_node_bounds.clone(), b_node_bounds.clone()],
            })),
        };

        let ray = Ray::new(Vec3 {x: 0.0, y: 0.5, z: 0.9}, Vec3::forward_rh());
        let mut t_range = Range {start: EPSILON, end: INFINITY};

        let (_, mat) = root.ray_cast(&ray, &mut t_range).unwrap();
        assert_eq!(mat, mat_b);
    }

    #[test]
    fn ray_cast_edge_case_flipped() {
        // This is the exact same case as above but with all the z values flipped except for the
        // separating plane. This causes the ray to go from the back to the front of the separating
        // plane.

        let mat_b = Arc::new(Material {
            diffuse: Rgb::red(),
            ..Material::default()
        });

        let trans_b = Mat4::scaling_3d(2.0)
            .rotated_x(-90f64.to_radians())
            .translated_3d((0.0, 1.2, 0.4));
        let node_b = FlatSceneNode::new(Geometry::new(FinitePlane, mat_b.clone()), trans_b);
        let b_node_bounds = Arc::new(NodeBounds {
            bounds: node_b.bounds(),
            node: node_b,
        });

        let mat_c = Arc::new(Material {
            diffuse: Rgb::blue(),
            ..Material::default()
        });
        assert_ne!(mat_b, mat_c);

        let trans_c = Mat4::scaling_3d(2.0)
            .rotated_x(-50f64.to_radians())
            .translated_3d((0.0, 0.0, 0.3));
        let node_c = FlatSceneNode::new(Geometry::new(FinitePlane, mat_c.clone()), trans_c);
        let c_node_bounds = Arc::new(NodeBounds {
            bounds: node_c.bounds(),
            node: node_c,
        });

        let root = KDTreeNode::Split {
            sep_plane: Plane {normal: Vec3::unit_z(), point: Vec3::zero()},
            bounds: vec![b_node_bounds.clone(), c_node_bounds.clone()].bounds(),
            front_nodes: Box::new(KDTreeNode::Leaf(KDLeaf {
                // leaf bounds do not matter currently
                bounds: BoundingBox::new(Vec3::zero(), Vec3::zero()),
                nodes: vec![c_node_bounds.clone(), b_node_bounds.clone()],
            })),
            back_nodes: Box::new(KDTreeNode::Leaf(KDLeaf {
                // leaf bounds do not matter currently
                bounds: BoundingBox::new(Vec3::zero(), Vec3::zero()),
                // Force tree to check C again by putting it first
                nodes: vec![c_node_bounds.clone()],
            })),
        };

        let ray = Ray::new(Vec3 {x: 0.0, y: 0.5, z: -0.9}, Vec3::back_rh());
        let mut t_range = Range {start: EPSILON, end: INFINITY};

        let (_, mat) = root.ray_cast(&ray, &mut t_range).unwrap();
        assert_eq!(mat, mat_b);
    }
}
