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

    let mat = Material {
        diffuse: 0.5.into(),
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
        root: &SceneNode::from(Geometry {
            prim: Sphere.into(),
            mat,
        }),
        lights: &[
            Light {
                position: cam.eye,
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                falloff: Default::default(),
            },
        ],
        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    image.draw(&scene, cam, |_, _| Rgb::black());

    image.save("test.png")
}
