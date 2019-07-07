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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::math::{EPSILON, INFINITY, Rgb, Mat4, Vec3};
    use crate::material::Material;
    use crate::scene::Geometry;
    use crate::primitive::FinitePlane;

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
            front_nodes: Box::new(KDTreeNode::Leaf {
                // leaf bounds do not matter currently
                bounds: BoundingBox::new(Vec3::zero(), Vec3::zero()),
                nodes: vec![c_node_bounds.clone()],
            }),
            back_nodes: Box::new(KDTreeNode::Leaf {
                // leaf bounds do not matter currently
                bounds: BoundingBox::new(Vec3::zero(), Vec3::zero()),
                // Force tree to check C again by putting it first
                nodes: vec![c_node_bounds.clone(), b_node_bounds.clone()],
            }),
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
            front_nodes: Box::new(KDTreeNode::Leaf {
                // leaf bounds do not matter currently
                bounds: BoundingBox::new(Vec3::zero(), Vec3::zero()),
                nodes: vec![c_node_bounds.clone(), b_node_bounds.clone()],
            }),
            back_nodes: Box::new(KDTreeNode::Leaf {
                // leaf bounds do not matter currently
                bounds: BoundingBox::new(Vec3::zero(), Vec3::zero()),
                // Force tree to check C again by putting it first
                nodes: vec![c_node_bounds.clone()],
            }),
        };

        let ray = Ray::new(Vec3 {x: 0.0, y: 0.5, z: -0.9}, Vec3::back_rh());
        let mut t_range = Range {start: EPSILON, end: INFINITY};

        let (_, mat) = root.ray_cast(&ray, &mut t_range).unwrap();
        assert_eq!(mat, mat_b);
    }
}
