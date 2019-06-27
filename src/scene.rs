use crate::math::{Mat4, Vec3, Rgb, Radians};
use crate::primitive::Primitive;
use crate::material::Material;
use crate::light::Light;

use crate::arena::{Arena, Handle};

#[derive(Debug)]
pub struct Scene {
    root: Handle<SceneNode>,
    lights: Vec<Light>,
    ambient: Rgb,
    nodes: Arena<SceneNode>,
    materials: Arena<Material>,
}

impl Default for Scene {
    fn default() -> Self {
        let mut nodes = Arena::default();
        Self {
            root: nodes.insert(SceneNode::default()),
            lights: Vec::new(),
            ambient: Default::default(),
            nodes,
            materials: Arena::default(),
        }
    }
}

impl Scene {
    pub fn new(root: SceneNode, ambient: Rgb) -> Self {
        let mut scene = Scene::default();
        let root = scene.add_node(root);

        Self {
            root,
            ambient,
            ..scene
        }
    }

    pub fn with_ambient(ambient: Rgb) -> Self {
        Self {
            ambient,
            ..Scene::default()
        }
    }

    pub fn root(&self) -> &SceneNode {
        self.nodes.get(self.root)
    }

    pub fn root_mut(&mut self) -> &mut SceneNode {
        self.nodes.get_mut(self.root)
    }

    pub fn lights(&self) -> impl Iterator<Item=&Light> {
        self.lights.iter()
    }

    pub fn ambient(&self) -> Rgb {
        self.ambient
    }

    pub fn node(&self, node: Handle<SceneNode>) -> &SceneNode {
        self.nodes.get(node)
    }

    pub fn material(&self, material: Handle<Material>) -> &Material {
        self.materials.get(material)
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light)
    }

    pub fn add_node(&mut self, node: SceneNode) -> Handle<SceneNode> {
        self.nodes.insert(node)
    }

    pub fn add_material(&mut self, material: Material) -> Handle<Material> {
        self.materials.insert(material)
    }
}

#[derive(Debug)]
pub struct Geometry {
    primitive: Primitive,
    material: Handle<Material>,
}

impl Geometry {
    pub fn new<P: Into<Primitive>>(primitive: P, material: Handle<Material>) -> Self {
        Self {
            primitive: primitive.into(),
            material,
        }
    }

    pub fn primitive(&self) -> &Primitive {
        &self.primitive
    }

    pub fn material<'a>(&self, scene: &'a Scene) -> &'a Material {
        scene.material(self.material)
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
    children: Vec<Handle<SceneNode>>,
}

impl From<Geometry> for SceneNode {
    fn from(geometry: Geometry) -> Self {
        Self {
            geometry: Some(geometry),
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
    pub fn children<'a>(&'a self, scene: &'a Scene) -> impl Iterator<Item=&'a SceneNode> + 'a {
        self.children.iter().cloned().map(move |child| scene.nodes.get(child))
    }

    /// Add the given child to this node
    pub fn add_child(&mut self, child: Handle<SceneNode>) {
        self.children.push(child)
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
