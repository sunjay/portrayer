//! Title: "The Temple of Computer Graphics"
//! Author: Sunjay Varma

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Sphere, Cube},
    material::{Material, WATER_REFRACTION_INDEX},
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb, Uv},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let mat_water = Arc::new(Material {
        diffuse: Rgb {r: 0.0, g: 0.0, b: 0.1},
        specular: Rgb {r: 0.5, g: 0.5, b: 0.5},
        shininess: 100.0,
        reflectivity: 0.9,
        glossy_side_length: 3.0,
        refraction_index: WATER_REFRACTION_INDEX,
        ..Material::default()
    });

    let mat_dirt = Arc::new(Material {
        diffuse: Rgb {r: 0.486902, g: 0.321985, b: 0.239812},
        ..Material::default()
    });

    let mat_red = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.059494, b: 0.108538},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let mat_green = Arc::new(Material {
        diffuse: Rgb {r: 0.201293, g: 0.8, b: 0.168034},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let scene = HierScene {
        root: SceneNode::from(vec![
            SceneNode::from(Geometry::new(Cube, mat_dirt.clone()))
                .scaled((40.0, 10.0, 20.0))
                .translated((0.0, -5.0, -19.0))
                .into(),

            SceneNode::from(Geometry::new(Cube, mat_water.clone()))
                .scaled((40.0, 10.0, 30.0))
                .translated((0.0, -5.0, 6.0))
                .into(),

            SceneNode::from(Geometry::new(Sphere, mat_red.clone()))
                .scaled(5.0)
                .translated((-6.0, 5.0, -5.0))
                .into(),
            SceneNode::from(Geometry::new(Sphere, mat_green.clone()))
                .scaled(5.0)
                .translated((6.0, 5.0, -5.0))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 0.0, y: 29.0, z: 10.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 5.0, 40.0).into(),
        center: (0.0, 5.0, -40.0).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(25.0),
    };

    // let mut image = RgbImage::new(1920, 1080);
    let mut image = RgbImage::new(533, 300);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.529, g: 0.808, b: 0.922} * (1.0 - uv.v) + Rgb::white() * uv.v);

    Ok(image.save("graphics-temple.png")?)
}
