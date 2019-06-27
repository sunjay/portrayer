//! A simple scene with five spheres

use std::io;

use portrayer::{
    scene::{Scene, SceneNode, Geometry},
    primitive::Sphere,
    material::Material,
    light::Light,
    render::Target,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb},
};
use image::RgbImage;

fn main() -> io::Result<()> {
    let mat1 = Material {
        diffuse: Rgb {r: 0.7, g: 1.0, b: 0.7},
        specular: Rgb {r: 0.5, g: 0.7, b: 0.5},
        shininess: 25.0,
        reflectivity: 0.0,
    };
    let mat2 = Material {
        diffuse: Rgb {r: 0.5, g: 0.5, b: 0.5},
        specular: Rgb {r: 0.5, g: 0.7, b: 0.5},
        shininess: 25.0,
        reflectivity: 0.0,
    };
    let mat3 = Material {
        diffuse: Rgb {r: 1.0, g: 0.6, b: 0.1},
        specular: Rgb {r: 0.5, g: 0.7, b: 0.5},
        shininess: 25.0,
        reflectivity: 0.0,
    };

    let scene = Scene {
        root: &SceneNode::from(vec![
            SceneNode::from(Geometry::new(Sphere, &mat1))
                .scaled(100.0)
                .translated((0.0, 0.0, -400.0)),

            SceneNode::from(Geometry::new(Sphere, &mat1))
                .scaled(150.0)
                .translated((200.0, 50.0, -100.0)),

            SceneNode::from(Geometry::new(Sphere, &mat2))
                .scaled(1000.0)
                .translated((0.0, -1200.0, -500.0)),

            SceneNode::from(Geometry::new(Sphere, &mat3))
                .scaled(50.0)
                .translated((-100.0, 25.0, -300.0)),

            SceneNode::from(Geometry::new(Sphere, &mat1))
                .scaled(25.0)
                .translated((0.0, 100.0, -250.0)),
        ]),
        lights: &[
            // white_light
            Light {
                position: Vec3 {x: -100.0, y: 150.0, z: 400.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                falloff: Default::default(),
            },
            // magenta_light
            Light {
                position: Vec3 {x: 400.0, y: 100.0, z: 150.0},
                color: Rgb {r: 0.7, g: 0.0, b: 0.7},
                falloff: Default::default(),
            },
        ],
        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 0.0, 800.0).into(),
        center: (0.0, 0.0, 0.0).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(50.0),
    };

    let mut image = RgbImage::new(256, 256);

    image.draw(&scene, cam,
        |_, y| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - y) + Rgb::blue() * y);

    image.save("simple.png")
}
