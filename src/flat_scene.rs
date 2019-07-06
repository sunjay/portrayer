use std::sync::Arc;
use std::ops::Range;
use std::collections::VecDeque;

use crate::math::{Mat4, Vec3Ext};
use crate::material::Material;
use crate::scene::{Scene, HierScene, Geometry};
use crate::ray::{RayCast, RayHit, Ray, RayIntersection};
use crate::bounding_box::{BoundingBox, Bounds};

/// A completely non-hierarchical representation of the scene. Note that this potentially uses
/// more memory since all structures that were previously benefiting from instancing will get
/// cloned. The process of converting a hierarchical scene to a flat scene assumes that the scene
/// is a tree. If any cycles do exist, the flattening structure will never terminate and will
/// consume all memory.
pub type FlatScene = Scene<Vec<FlatSceneNode>>;

impl<'a> From<&'a HierScene> for FlatScene {
    fn from(hier_scene: &'a HierScene) -> Self {
        // Performing a breadth first traversal through the tree
        // Note that no cycle checking occurs here. We are assuming that the scene is a tree.
        let mut nodes = Vec::new();
        // Contains (parent transform, node) pairs
        let mut remaining = VecDeque::new();
        remaining.push_back((Mat4::identity(), hier_scene.root.clone()));

        while let Some((parent_trans, node)) = remaining.pop_front() {
            // The total transformation so far
            let total_trans = parent_trans * node.trans();

            if let Some(geometry) = node.geometry() {
                nodes.push(FlatSceneNode::new(geometry.clone(), total_trans));
            }

            for child in node.children() {
                remaining.push_back((total_trans, child.clone()));
            }
        }

        Self {
            root: nodes,
            lights: hier_scene.lights.clone(),
            ambient: hier_scene.ambient,
        }
    }
}

/// A scene node with no hierarchical structure
#[derive(Debug)]
pub struct FlatSceneNode {
    /// The geometry stored at this node
    ///
    /// Node must contain geometry since it would be useless otherwise.
    geometry: Geometry,
    /// The affine transform of this node (model space to world space)
    trans: Mat4,
    /// The inverse of the affine transform of this node
    invtrans: Mat4,
    /// The inverse transpose of trans, used for transforming normals
    normal_trans: Mat4,
}

impl Bounds for FlatSceneNode {
    fn bounds(&self) -> BoundingBox {
        let prim_bounds = self.geometry.primitive.bounds();

        self.trans * prim_bounds
    }
}

impl RayCast for FlatSceneNode {
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)> {
        // Take the ray from its current coordinate system and put it into the local coordinate
        // system of the node
        let local_ray = ray.transformed(self.inverse_trans());

        // These will be used to transform the hit point and normal back into the
        // previous coordinate system
        let trans = self.trans();
        let normal_trans = self.normal_trans();

        // Check if the ray intersects this node's geometry
        let Geometry {primitive, material} = &self.geometry;
        match primitive.ray_hit(&local_ray, t_range) {
            Some(mut hit) => {
                // Bring the found hit point back into the right coordinate system
                hit.hit_point = hit.hit_point.transformed_point(trans);
                hit.normal = hit.normal.transformed_direction(normal_trans);

                // Only allow further intersections if they are closer to the ray origin
                // than this one
                t_range.end = hit.ray_parameter;

                Some((hit, material.clone()))
            },
            None => None,
        }
    }
}

impl FlatSceneNode {
    /// Creates a new flat scene node with the given geometry and transformation
    pub fn new(geometry: Geometry, trans: Mat4) -> Self {
        let invtrans = trans.inverted();
        let normal_trans = invtrans.transposed();

        Self {geometry, trans, invtrans, normal_trans}
    }

    /// Return the geometry stored at this node
    pub fn geometry(&self) -> &Geometry {
        &self.geometry
    }

    /// Returns the transformation matrix of this node
    pub fn trans(&self) -> Mat4 {
        self.trans
    }

    /// Returns the inverse of the transformation matrix of this node
    pub fn inverse_trans(&self) -> Mat4 {
        self.invtrans
    }

    /// Returns the transform that should be used on normals
    ///
    /// This is the same as inverse_trans().transposed()
    pub fn normal_trans(&self) -> Mat4 {
        self.normal_trans
    }
}
