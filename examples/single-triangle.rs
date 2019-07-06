//! A simple scene with five spheres

use std::io;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::Triangle,
    material::Material,
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;

fn main() -> io::Result<()> {
    let mat1 = Arc::new(Material {
        diffuse: Rgb {r: 0.541, g: 0.169, b: 0.886},
        specular: Rgb {r: 0.5, g: 0.7, b: 0.5},
        shininess: 25.0,
        ..Material::default()
    });

    let triangle = Triangle::flat(
        Vec3 {x: -1.0, y: 0.0, z: 0.0},
        Vec3 {x: 1.0, y: 0.0, z: 0.0},
        Vec3 {x: 0.0, y: 1.5, z: 0.0},
    );

    let scene = HierScene {
        root: SceneNode::from(vec![
            SceneNode::from(Geometry::new(triangle, mat1.clone()))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 1.0, y: 1.0, z: 10.0},
                color: Rgb {r: 0.5, g: 0.5, b: 0.5},
                falloff: Default::default(),
            },
        ],
        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 0.5, 4.0).into(),
        center: (0.0, 0.5, 0.0).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(50.0),
    };

    let mut image = RgbImage::new(640, 480);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    image.save("single-triangle.png")
}
