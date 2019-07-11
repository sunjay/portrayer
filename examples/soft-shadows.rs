//! Demonstrates the glossy reflection feature

use std::io;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cone, Cube, Cylinder, FinitePlane},
    material::Material,
    light::{Light, Parallelogram},
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;

fn main() -> io::Result<()> {
    let mat_cylinder = Arc::new(Material {
        diffuse: Rgb {r: 0.004424, g: 0.075609, b: 0.8},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_cone = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.015794, b: 0.022275},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_cube = Arc::new(Material {
        diffuse: Rgb {r: 0.0, g: 0.8, b: 0.000563},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_floor = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.767947, b: 0.647146},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let scene = HierScene {
        root: SceneNode::from(vec![
            // Walls
            SceneNode::from(Geometry::new(FinitePlane, mat_floor.clone()))
                .scaled(20.0)
                .into(),
            SceneNode::from(Geometry::new(FinitePlane, mat_floor.clone()))
                .scaled(20.0)
                .rotated_x(Radians::from_degrees(90.0))
                .translated((0.0, 10.0, -10.0))
                .into(),
            SceneNode::from(Geometry::new(FinitePlane, mat_floor.clone()))
                .scaled(20.0)
                .rotated_z(Radians::from_degrees(-90.0))
                .translated((-10.0, 10.0, 0.0))
                .into(),

            // Objects
            SceneNode::from(Geometry::new(Cylinder, mat_cylinder.clone()))
                .scaled((0.5, 2.8, 0.5))
                .translated((-4.0, 1.4, 1.0))
                .into(),
            SceneNode::from(Geometry::new(Cone, mat_cone.clone()))
                .scaled((2.0, 4.0, 2.0))
                .translated((0.0, 2.0, 0.0))
                .into(),
            SceneNode::from(Geometry::new(Cube, mat_cube.clone()))
                .scaled(2.0)
                .translated((4.0, 1.0, -3.0))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 10.076245, y: 6.903862, z: 9.994546},
                color: Rgb {r: 0.5, g: 0.5, b: 0.5},
                ..Light::default()
            },

            Light {
                position: Vec3 {x: -9.0, y: 6.0, z: -2.3},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                area: Parallelogram {
                    a: Vec3 {x: 0.0, y: 2.0, z: 0.0},
                    b: Vec3 {x: 0.0, y: 0.0, z: 2.0},
                },
                ..Light::default()
            },
            Light {
                position: Vec3 {x: 0.0, y: 6.0, z: -9.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                area: Parallelogram {
                    a: Vec3 {x: 2.5, y: 0.0, z: 0.0},
                    b: Vec3 {x: 0.0, y: 1.5, z: 0.0},
                },
                ..Light::default()
            },
        ],

        // Lighting really matters in this scene, so no ambient
        ambient: Rgb::black(),
    };

    let cam = CameraSettings {
        eye: (15.829148, 10.746838, 14.910006).into(),
        center: (-6.323831, -4.39239, -5.971786).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(30.0),
    };

    let mut image = RgbImage::new(910, 512);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    image.save("soft-shadows.png")
}
