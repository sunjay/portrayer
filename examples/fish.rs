//! A simple test scene that tests mesh texture mapping

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Mesh, MeshData, Shading},
    material::Material,
    texture::{Texture, ImageTexture},
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let fish_skin = Arc::new(Texture::from(ImageTexture::open("assets/fish.png")?));
    let mat_fish = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.8, b: 0.8},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        texture: Some(fish_skin.clone()),
        ..Material::default()
    });

    let fish_model = Arc::new(MeshData::load_obj("assets/fish.obj")?);

    let scene = HierScene {
        root: SceneNode::from(vec![
            SceneNode::from(Geometry::new(Mesh::new(fish_model.clone(), Shading::Smooth), mat_fish.clone()))
                .rotated_y(Radians::from_degrees(30.0))
                .into(),

            SceneNode::from(Geometry::new(Mesh::new(fish_model.clone(), Shading::Smooth), mat_fish.clone()))
                .rotated_y(Radians::from_degrees(210.0))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 0.0, y: 0.0, z: 10.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 0.0, 11.0).into(),
        center: (0.0, 0.0, 0.0).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(25.0),
    };

    let mut image = RgbImage::new(910, 512);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save("fish.png")?)
}
