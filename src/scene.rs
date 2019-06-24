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
    /// Any child nodes that are hierarchically "underneath" this node
    children: Vec<SceneNode>,
}
