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
    let mut image = RgbImage::new(256, 256);

    let mat1 = Material {
        diffuse: 0.3.into(),
        specular: 0.8.into(),
        shininess: 10.0,
        reflectivity: 0.0,
    };

    let mat2 = Material {
        diffuse: (0.2, 0.5, 0.5).into(),
        specular: 0.8.into(),
        shininess: 10.0,
        reflectivity: 0.0,
    };

    let cam = CameraSettings {
        eye: (0.0, 0.0, 3.0).into(),
        center: (0.0, 0.0, 0.0).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(50.0),
    };

    let scene = Scene {
        root: &SceneNode::from(vec![
            SceneNode::from(Geometry::new(Sphere, mat1))
                .scaled(2.0)
                .translated((0.0, 2.0, 0.0)),

            SceneNode::from(Geometry::new(Sphere, mat2))
                .scaled(1.5)
                .translated((-1.0, 0.0, 0.0)),
        ]),
        lights: &[
            Light {
                position: cam.eye,
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                falloff: Default::default(),
            },
        ],
        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    image.draw(&scene, cam,
        |_, y| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - y) + Rgb::blue() * y);

    image.save("test.png")
}
