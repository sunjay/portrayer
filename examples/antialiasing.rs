//! Generates multiple test images with different numbers of samples

use std::env;
use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Mesh, MeshData, Shading},
    material::Material,
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let mat_monkey = Arc::new(Material {
        diffuse: Rgb {r: 0.961, g: 0.573, b: 0.259},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let monkey_mesh = Arc::new(MeshData::load_obj("assets/monkey.obj")?);

    let scene = HierScene {
        root: SceneNode::from(vec![
            SceneNode::from(Geometry::new(Mesh::new(monkey_mesh.clone(), Shading::Flat), mat_monkey.clone()))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 0.0, y: 0.0, z: 10.0},
                color: Rgb {r: 0.5, g: 0.5, b: 0.5},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 0.0, 6.5).into(),
        center: (0.0, 0.0, 0.0).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(20.0),
    };

    let mut image = RgbImage::new(300, 250);

    for samples in &[1, 32] {
        println!("Rendering with {} samples", samples);
        env::set_var("SAMPLES", samples.to_string());

        image.render::<RenderProgress, _>(&scene, cam,
            |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

        image.save(format!("antialiasing_{}.png", samples))?;
    }

    Ok(())
}
