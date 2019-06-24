use crate::math::Mat4;
use crate::primitive::Primitive;

#[derive(Debug, Default)]
pub struct SceneNode {
    prim: Option<Primitive>,
    trans: Mat4,
    children: Vec<SceneNode>,
}

impl From<Primitive> for SceneNode {
    fn from(prim: Primitive) -> Self {
        Self {
            prim: Some(prim),
            ..Default::default()
        }
    }
}
