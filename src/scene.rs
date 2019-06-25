use crate::math::{Mat4, Vec3, Rgb};
use crate::primitive::Primitive;
use crate::material::Material;
use crate::light::Light;

#[derive(Debug)]
pub struct Scene<'a> {
    pub root: &'a SceneNode,
    pub lights: &'a [Light],
    pub ambient: Rgb,
}

#[derive(Debug)]
pub struct Geometry {
    pub prim: Primitive,
    pub mat: Material,
}

impl Geometry {
    pub fn new<P: Into<Primitive>>(prim: P, mat: Material) -> Self {
        Self {
            prim: prim.into(),
            mat,
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
    children: Vec<SceneNode>,
}

impl From<Geometry> for SceneNode {
    fn from(geometry: Geometry) -> Self {
        Self {
            geometry: Some(geometry),
            ..Default::default()
        }
    }
}

impl From<Vec<SceneNode>> for SceneNode {
    fn from(children: Vec<SceneNode>) -> Self {
        Self {
            children,
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
    pub fn children(&self) -> &[SceneNode] {
        &*self.children
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

    /// Update the transformation matrix to the given value
    pub fn set_transform(&mut self, transform: Mat4) {
        self.trans = transform;
        self.invtrans = transform.inverted();
        self.normal_trans = self.invtrans.transposed();
    }
}
