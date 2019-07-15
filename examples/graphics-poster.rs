//! The poster on the wall of the "Monkeys Making Better Monkeys" scene

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Mesh, MeshData, Shading},
    material::{Material, OPTICAL_GLASS_REFRACTION_INDEX},
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let mat_glass = Arc::new(Material {
        diffuse: Rgb {r: 0.003638, g: 0.017153, b: 0.048247},
        specular: Rgb {r: 0.5, g: 0.5, b: 0.5},
        shininess: 100.0,
        reflectivity: 0.8,
        glossy_side_length: 0.5,
        refraction_index: OPTICAL_GLASS_REFRACTION_INDEX,
        ..Material::default()
    });

    let mat_cow = Arc::new(Material {
        diffuse: Rgb {r: 0.725682, g: 0.501253, b: 0.8},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let dodeca_model = Arc::new(MeshData::load_obj("assets/dodeca.obj")?);
    let cow_model = Arc::new(MeshData::load_obj("assets/cow.obj")?);

    let scene = HierScene {
        root: SceneNode::from(vec![
            SceneNode::from(Geometry::new(Mesh::new(dodeca_model, Shading::Flat), mat_glass.clone()))
                .rotated_y(Radians::from_degrees(90.0))
                .into(),

            SceneNode::from(Geometry::new(Mesh::new(cow_model, Shading::Smooth), mat_cow.clone()))
                .scaled(0.24)
                .rotated_y(Radians::from_degrees(-60.0))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 1.33223, y: 4.297232, z: 3.473453},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
            // Need a light inside the mesh to illuminate the cow
            Light {
                position: Vec3 {x: 0.8, y: 0.806596, z: 0.9},
                color: Rgb {r: 0.3, g: 0.3, b: 0.3},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (4.482203, 3.038775, 4.350142).into(),
        center: (-7.387217, -4.572944, -6.838186).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(35.0),
    };

    // let mut image = RgbImage::new(1080, 1080);
    let mut image = RgbImage::new(256, 256);

    image.render::<RenderProgress, _>(&scene, cam, |_| Rgb::white());

    Ok(image.save("graphics-poster.png")?)
}
