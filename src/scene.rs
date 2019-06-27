use std::sync::Arc;

use crate::math::{Mat4, Vec3, Rgb, Radians};
use crate::primitive::Primitive;
use crate::material::Material;
use crate::light::Light;

#[derive(Debug)]
pub struct Scene {
    pub root: Arc<SceneNode>,
    pub lights: Vec<Light>,
    pub ambient: Rgb,
}

#[derive(Debug)]
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
    /// The affine transform of this node
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

impl SceneNode {
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
    pub fn children(&self) -> impl Iterator<Item=&Arc<SceneNode>> {
        self.children.iter()
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
