use crate::math::{Mat4, Rgb};
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
}
