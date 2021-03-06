use std::sync::Arc;
use std::ops::Range;

use crate::math::{Mat4, Vec3, Vec3Ext, Rgb, Radians};
use crate::ray::{RayCast, Ray, RayIntersection, RayHit};
use crate::primitive::Primitive;
use crate::material::Material;
use crate::light::Light;

/// A hierarchical scene
pub type HierScene = Scene<Arc<SceneNode>>;

#[derive(Debug)]
pub struct Scene<R> {
    pub root: R,
    pub lights: Vec<Light>,
    pub ambient: Rgb,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    pub primitive: Primitive,
    pub material: Arc<Material>,
}

impl Geometry {
    pub fn new<P: Into<Primitive>>(primitive: P, material: Arc<Material>) -> Self {
        Self {
            primitive: primitive.into(),
            material,
        }
    }
}

#[derive(Debug, Default)]
pub struct SceneNode {
    /// The geometry stored at this node (if any)
    geometry: Option<Geometry>,
    /// The affine transform of this node (model space to world space)
    trans: Mat4,
    /// The inverse of the affine transform of this node
    invtrans: Mat4,
    /// The inverse transpose of trans, used for transforming normals
    normal_trans: Mat4,
    /// Any child nodes that are hierarchically "underneath" this node
    children: Vec<Arc<SceneNode>>,
}

// Create a node with the given geometry
impl From<Geometry> for SceneNode {
    fn from(geometry: Geometry) -> Self {
        Self {
            geometry: Some(geometry),
            ..Default::default()
        }
    }
}

// Create a node from multiple children
impl From<Vec<Arc<SceneNode>>> for SceneNode {
    fn from(children: Vec<Arc<SceneNode>>) -> Self {
        Self {
            children,
            ..Default::default()
        }
    }
}

// Create a node from a single child
impl From<Arc<SceneNode>> for SceneNode {
    fn from(child: Arc<SceneNode>) -> Self {
        Self {
            children: vec![child],
            ..Default::default()
        }
    }
}

/// For casting a ray through a hierarchical scene
impl RayCast for SceneNode {
    fn ray_cast(&self, ray: &Ray, t_range: &mut Range<f64>) -> Option<(RayIntersection, Arc<Material>)> {
        // Take the ray from its current coordinate system and put it into the local coordinate
        // system of the current node
        let local_ray = ray.transformed(self.inverse_trans());

        // These will be used to transform the hit point and normal back into the
        // previous coordinate system
        let trans = self.trans();
        let normal_trans = self.normal_trans();

        // The resulting hit and material (initially None)
        let mut hit_mat = None;

        // Check if the ray intersects this node's geometry (if any)
        if let Some(Geometry {primitive, material}) = self.geometry() {
            if let Some(mut hit) = primitive.ray_hit(&local_ray, t_range) {
                hit.hit_point = hit.hit_point.transformed_point(trans);
                hit.normal = hit.normal.transformed_direction(normal_trans);

                // Only allow further intersections if they are closer to the ray origin
                // than this one
                t_range.end = hit.ray_parameter;

                hit_mat = Some((hit, material.clone()));
            }
        }

        // Recurse into children and attempt to find a closer match
        if let Some((mut child_hit, child_mat)) = self.children().ray_cast(&local_ray, t_range) {
            child_hit.hit_point = child_hit.hit_point.transformed_point(trans);
            child_hit.normal = child_hit.normal.transformed_direction(normal_trans);

            // No need to set t_range.end since it is set in the recursive base case of this method

            hit_mat = Some((child_hit, child_mat));
        }

        hit_mat
    }
}

impl SceneNode {
    /// Return the geometry stored at this node (if any)
    pub fn geometry(&self) -> Option<&Geometry> {
        self.geometry.as_ref()
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

    /// For iterating over the children of this node
    pub fn children(&self) -> &[Arc<SceneNode>] {
        &self.children
    }

    /// Add the given child to this node and return the updated node
    pub fn with_child<C: Into<Arc<SceneNode>>>(mut self, child: C) -> Self {
        self.children.push(child.into());
        self
    }

    /// Add the given children to this node and return the updated node
    pub fn with_children<I: IntoIterator<Item=Arc<SceneNode>>>(mut self, children: I) -> Self {
        self.children.extend(children);
        self
    }

    /// Scale the node by the given vector and return the node
    pub fn scaled<V: Into<Vec3>>(mut self, scale: V) -> Self {
        self.set_transform(self.trans.scaled_3d(scale));
        self
    }

    /// Translate the node by the given vector and return the node
    pub fn translated<V: Into<Vec3>>(mut self, translation: V) -> Self {
        self.set_transform(self.trans.translated_3d(translation));
        self
    }

    /// Rotate about the x-axis, then z-axis, then y-axis by the given angles
    ///
    /// Useful for converting from Blender XYZ angles to our right-handed coordinate system
    pub fn rotated_xzy<V: Into<vek::Vec3<Radians>>>(self, angles: V) -> Self {
        let vek::Vec3 {x, y, z} = angles.into();
        self.rotated_x(x).rotated_z(z).rotated_y(y)
    }

    /// Rotate about the x-axis by the given angle
    pub fn rotated_x(mut self, angle: Radians) -> Self {
        self.set_transform(self.trans.rotated_x(angle.get()));
        self
    }

    /// Rotate about the y-ayis by the given angle
    pub fn rotated_y(mut self, angle: Radians) -> Self {
        self.set_transform(self.trans.rotated_y(angle.get()));
        self
    }

    /// Rotate about the z-azis by the given angle
    pub fn rotated_z(mut self, angle: Radians) -> Self {
        self.set_transform(self.trans.rotated_z(angle.get()));
        self
    }

    /// Update the transformation matrix to the given value
    pub fn set_transform(&mut self, transform: Mat4) {
        self.trans = transform;
        self.invtrans = transform.inverted();
        self.normal_trans = self.invtrans.transposed();
    }
}
