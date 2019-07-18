use std::sync::Arc;
use std::ops::Range;

use crate::math::EPSILON;
use crate::material::Material;
use crate::primitive::{InfinitePlane, PlaneSide};
use crate::bounding_box::BoundingBox;
use crate::ray::{RayCast, RayHit, Ray, RayIntersection};

use super::{KDLeaf, NodeBounds};

#[derive(Debug, PartialEq)]
pub(crate) enum KDTreeNode<T> {
    Split {
        /// The separating plane that divides the children
        sep_plane: InfinitePlane,
        /// A bounding box that encompases all of the nodes in this node
        bounds: BoundingBox,
        /// The nodes in front of the separating plane (in the direction of the plane normal)
        front_nodes: Box<KDTreeNode<T>>,
        /// The nodes behind the separating plane (in the direction opposite to the plane normal)
        back_nodes: Box<KDTreeNode<T>>,
    },
    Leaf(KDLeaf<T>),
}

impl<T: RayCast> RayCast for KDTreeNode<T> {
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)> {
        self.ray_cast_impl(ray, t_range, self.extent(), &mut RayCast::ray_cast)
    }
}

impl<T: RayHit> RayHit for KDTreeNode<T> {
    fn ray_hit(&self, ray: &Ray, init_t_range: &Range<f64>) -> Option<RayIntersection> {
        // Need to emulate RayCast here and modify a range so that we can ensure we get the nearest
        // intersection possible. This is also important because ray_cast_impl expects the given
        // function to provide the same guarantees as RayCast about updating the t_range.
        let mut t_range = init_t_range.clone();
        self.ray_cast_impl(ray, &mut t_range, self.extent(), &mut |nodes, ray, t_range| {
            match nodes.ray_hit(ray, t_range) {
                Some(hit) => {
                    // Only allow further intersections if they are closer to the ray origin
                    // than this one
                    t_range.end = hit.ray_parameter;
                    Some(hit)
                },
                None => None,
            }
        })
    }
}

impl<T> KDTreeNode<T> {
    pub(in super) fn bounds(&self) -> &BoundingBox {
        use KDTreeNode::*;
        match self {
            Split {bounds, ..} |
            Leaf(KDLeaf {bounds, ..}) => bounds,
        }
    }

    fn extent(&self) -> f64 {
        self.bounds().extent()
    }

    fn ray_cast_impl<F, R>(
        &self,
        ray: &Ray,
        t_range: &mut Range<f64>,
        extent: f64,
        cast_ray: &mut F,
    ) -> Option<R>
        where F: FnMut(&[Arc<NodeBounds<T>>], &Ray, &mut Range<f64>) -> Option<R> {
        // To find the t value of the plane intersection, we exploit the fact that the plane is
        // axis-aligned and thus we already know one of the components of the hit_point that would
        // be returned from any other call to ray_hit.
        //
        // The ray_hit implementation of InfinitePlane suffers from some numerical issues when the
        // ray is right on the plane itself. This isn't typical for most scenes, but it can easily
        // happen in the k-d tree when the separating plane cross the eye point.
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
            sep_plane: &InfinitePlane,
            ray: &Ray,
            t_range: &Range<f64>,
        ) -> Option<f64> {
            // Multiplying by the normal will set two components to zero and sum() will
            // let us fish out the non-zero value.
            let plane_value = (sep_plane.normal * sep_plane.point).sum();
            let ray_origin = (sep_plane.normal * ray.origin()).sum();
            let ray_direction = (sep_plane.normal * ray.direction()).sum();

            // Need to add t_min because we used ray_start as the ray origin above
            let t = (plane_value - ray_origin) / ray_direction;

            // Must always ensure that we respect t_range or things can go *very* wrong
            if t_range.contains(&t) {
                Some(t)
            } else {
                None
            }
        }

        use KDTreeNode::*;
        match self {
            Leaf(KDLeaf {nodes, ..}) => cast_ray(&nodes[..], ray, t_range),
            Split {sep_plane, front_nodes, back_nodes, ..} => {
                // A value of t large enough that the point on the ray for this t would be well
                // beyond the extent of the scene. Need to add to t_range.start because otherwise
                // the bounds extent may not be enough.
                let t_max = t_range.start + extent;
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
                    (Front, Front) => front_nodes.ray_cast_impl(ray, t_range, extent, cast_ray),

                    // Ray segment lies entirely on back side of the separating plane
                    (Back, Back) => back_nodes.ray_cast_impl(ray, t_range, extent, cast_ray),

                    // Ray segment goes through the front nodes, then the back nodes
                    (Front, Back) => {
                        // Must ensure that any found intersection is actually on the checked side
                        // of the plane or else we can get incorrect results. We can do this by
                        // limiting the t_range based on the value of t for which the ray
                        // intersects the plane.

                        let plane_t = ray_hit_axis_aligned_plane(sep_plane, ray, t_range)
                            .expect("bug: ray should definitely hit infinite plane");

                        // Only going to continue with this range if it hits
                        let mut front_t_range = Range {start: t_range.start, end: plane_t};
                        match front_nodes.ray_cast_impl(ray, &mut front_t_range, extent, cast_ray) {
                            Some(hit_mat) => {
                                *t_range = front_t_range;
                                Some(hit_mat)
                            },
                            None => {
                                // Only going to continue with this range if it hits
                                let mut back_t_range = Range {start: plane_t, end: t_range.end};
                                match back_nodes.ray_cast_impl(ray, &mut back_t_range, extent, cast_ray) {
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

                        let plane_t = ray_hit_axis_aligned_plane(sep_plane, ray, t_range)
                            .expect("bug: ray should definitely hit infinite plane");

                        // Only going to continue with this range if it hits
                        let mut back_t_range = Range {start: t_range.start, end: plane_t};
                        match back_nodes.ray_cast_impl(ray, &mut back_t_range, extent, cast_ray) {
                            Some(hit_mat) => {
                                *t_range = back_t_range;
                                Some(hit_mat)
                            },
                            None => {
                                // Only going to continue with this range if it hits
                                let mut front_t_range = Range {start: plane_t, end: t_range.end};
                                match front_nodes.ray_cast_impl(ray, &mut front_t_range, extent, cast_ray) {
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
    use crate::bounding_box::Bounds;
    use crate::flat_scene::FlatSceneNode;
    use crate::material::Material;
    use crate::scene::Geometry;
    use crate::primitive::Plane;

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
        let node_b = FlatSceneNode::new(Geometry::new(Plane, mat_b.clone()), trans_b);
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
        let node_c = FlatSceneNode::new(Geometry::new(Plane, mat_c.clone()), trans_c);
        let c_node_bounds = Arc::new(NodeBounds {
            bounds: node_c.bounds(),
            node: node_c,
        });

        let root = KDTreeNode::Split {
            sep_plane: InfinitePlane {normal: Vec3::unit_z(), point: Vec3::zero()},
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
        let node_b = FlatSceneNode::new(Geometry::new(Plane, mat_b.clone()), trans_b);
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
        let node_c = FlatSceneNode::new(Geometry::new(Plane, mat_c.clone()), trans_c);
        let c_node_bounds = Arc::new(NodeBounds {
            bounds: node_c.bounds(),
            node: node_c,
        });

        let root = KDTreeNode::Split {
            sep_plane: InfinitePlane {normal: Vec3::unit_z(), point: Vec3::zero()},
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
