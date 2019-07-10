//! Demonstrates the glossy reflection feature

use std::io;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Sphere, Cube},
    material::Material,
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;

fn main() -> io::Result<()> {
    let non_glossy_ball = Arc::new(Material {
        diffuse: Rgb {r: 0.362595, g: 0.8, b: 0.424713},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 100.0,
        reflectivity: 0.4,
        ..Material::default()
    });
    let glossy_ball = Arc::new(Material {
        glossy_side_length: 2.0,
        ..(*non_glossy_ball).clone()
    });
    let center_ball = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.0, b: 0.023362},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });
    let table = Arc::new(Material {
        diffuse: Rgb {r: 1.0, g: 0.6, b: 0.1},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let scene = HierScene {
        root: SceneNode::from(vec![
            SceneNode::from(Geometry::new(Sphere, non_glossy_ball.clone()))
                .translated((-1.0, 1.3, 0.0))
                .into(),
            SceneNode::from(Geometry::new(Sphere, glossy_ball.clone()))
                .translated((1.0, 1.3, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Sphere, center_ball.clone()))
                .scaled(0.5)
                .translated((0.0, 0.8, 1.8))
                .into(),

            SceneNode::from(Geometry::new(Cube, table.clone()))
                .scaled((10.0, 0.6, 5.0))
                .into(),
        ]).into(),
        lights: vec![
            Light {
                position: Vec3 {x: 0.0, y: 6.0, z: 3.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                falloff: Default::default(),
            },

            Light {
                position: Vec3 {x: 0.0, y: 1.0, z: 12.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                falloff: Default::default(),
            },
        ],
        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 2.562834, 8.863271).into(),
        center: (0.0, -1.083779, -11.817695).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(20.0),
    };

    let mut image = RgbImage::new(910, 512);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    image.save("glossy-reflection.png")
}
