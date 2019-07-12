//! Demonstrates the glossy reflection feature

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, FinitePlane, Mesh, MeshData, Shading},
    material::Material,
    light::{Light, Parallelogram},
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let mat_cow = Arc::new(Material {
        diffuse: Rgb {r: 0.37168, g: 0.236767, b: 0.692066},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_wall_floor = Arc::new(Material {
        diffuse: Rgb {r: 0.627459, g: 0.8, b: 0.589836},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let cow_mesh = Arc::new(MeshData::load_obj("assets/cow.obj")?);

    let scene = HierScene {
        root: SceneNode::from(vec![
            // Walls + Floor
            SceneNode::from(Geometry::new(FinitePlane, mat_wall_floor.clone()))
                .scaled(30.0)
                .into(),
            SceneNode::from(Geometry::new(Cube, mat_wall_floor.clone()))
                .scaled((0.2, 20.0, 20.0))
                .translated((0.0, 8.0, 8.0))
                .into(),
            SceneNode::from(Geometry::new(Cube, mat_wall_floor.clone()))
                .scaled((30.0, 30.0, 0.4))
                .translated((0.0, 8.0, -2.0))
                .into(),

            // Objects
            SceneNode::from(Geometry::new(Mesh::new(cow_mesh.clone(), Shading::Smooth), mat_cow.clone()))
                .scaled(0.5)
                .rotated_y(Radians::from_degrees(-15.0))
                .translated((-4.2, 1.8, 4.0))
                .into(),
            SceneNode::from(Geometry::new(Mesh::new(cow_mesh.clone(), Shading::Smooth), mat_cow.clone()))
                .scaled(0.5)
                .rotated_y(Radians::from_degrees(195.0))
                .translated((4.2, 1.8, 4.0))
                .into(),
        ]).into(),

        lights: vec![
            // Left - Point Light
            Light {
                position: Vec3 {x: -2.0, y: 2.0, z: 16.0},
                color: Rgb {r: 0.5, g: 0.5, b: 0.5},
                ..Light::default()
            },

            // Right - Area Light
            Light {
                position: Vec3 {x: 2.0, y: 2.0, z: 16.0},
                color: Rgb {r: 0.5, g: 0.5, b: 0.5},
                area: Parallelogram {
                    a: Vec3 {x: 0.0, y: 1.0, z: 0.0},
                    b: Vec3 {x: 1.0, y: 0.0, z: 0.0},
                },
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 5.04746, 24.827951).into(),
        center: (0.012231, -0.459716, -15.800501).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(25.0),
    };

    let mut image = RgbImage::new(910, 512);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save("soft-shadows.png")?)
}
