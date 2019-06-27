//! Test for hierarchical ray-tracers.

use std::error::Error;
use std::path::Path;

use portrayer::{
    scene::{Scene, SceneNode, Geometry},
    primitive::{Sphere, Mesh, Cube},
    material::Material,
    light::Light,
    render::Target,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let gold = Material {
        diffuse: Rgb {r: 0.9, g: 0.8, b: 0.4},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.4},
        shininess: 25.0,
        reflectivity: 0.0,
    };
    let grass = Material {
        diffuse: Rgb {r: 0.1, g: 0.7, b: 0.1},
        specular: Rgb {r: 0.0, g: 0.0, b: 0.0},
        shininess: 0.0,
        reflectivity: 0.0,
    };
    let blue = Material {
        diffuse: Rgb {r: 0.7, g: 0.6, b: 1.0},
        specular: Rgb {r: 0.5, g: 0.4, b: 0.8},
        shininess: 25.0,
        reflectivity: 0.0,
    };

    let plane = &tobj::load_obj(Path::new("assets/plane.obj"))?.0[0].mesh;
    let dodeca = &tobj::load_obj(Path::new("assets/dodeca.obj"))?.0[0].mesh;

    // The arc
    let arc = SceneNode::from(vec![
        SceneNode::from(Geometry::new(Cube, &gold))
            .scaled((0.8, 4.0, 0.8))
            .translated((-2.0, 2.0, 0.0)),

        SceneNode::from(Geometry::new(Cube, &gold))
            .scaled((0.8, 4.0, 0.8))
            .translated((2.0, 2.0, 0.0)),

        SceneNode::from(Geometry::new(Sphere, &gold))
            .scaled((4.0, 0.6, 0.6))
            .translated((0.0, 4.0, 0.0)),
    ]).translated((0.0, 0.0, -10.0)).rotated_y(Radians::from_degrees(60.0));

    // The floor
    let floor = SceneNode::from(Geometry::new(Mesh::from(plane), &grass))
        .scaled(30.0);

    // Central "sphere"
    let poly = SceneNode::from(Geometry::new(Mesh::from(dodeca), &blue))
        .translated((-2.0, 1.618034, 0.0));

    let scene = Scene {
        root: &SceneNode::from(vec![arc, floor, poly])
            .rotated_x(Radians::from_degrees(23.0))
            .translated((6.0, -2.0, -15.0)),
        lights: &[
            // l1
            Light {
                position: Vec3 {x: 200.0, y: 200.0, z: 400.0},
                color: Rgb {r: 0.8, g: 0.8, b: 0.8},
                falloff: Default::default(),
            },
            // l2
            Light {
                position: Vec3 {x: 0.0, y: 5.0, z: -20.0},
                color: Rgb {r: 0.4, g: 0.4, b: 0.8},
                falloff: Default::default(),
            },
        ],
        ambient: Rgb {r: 0.4, g: 0.4, b: 0.4},
    };

    let cam = CameraSettings {
        eye: (0.0, 0.0, 0.0).into(),
        center: (0.0, 0.0, -1.0).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(50.0),
    };

    let mut image = RgbImage::new(256, 256);

    image.draw(&scene, cam,
        |_, y| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - y) + Rgb::blue() * y);

    Ok(image.save("hier.png")?)
}
