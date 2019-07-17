//! Test for hierarchical ray-tracers.

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Sphere, Mesh, MeshData, Shading, Cube},
    material::Material,
    light::Light,
    render::Image,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};

fn main() -> Result<(), Box<dyn Error>> {
    let gold = Arc::new(Material {
        diffuse: Rgb {r: 0.9, g: 0.8, b: 0.4},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.4},
        shininess: 25.0,
        ..Material::default()
    });
    let grass = Arc::new(Material {
        diffuse: Rgb {r: 0.1, g: 0.7, b: 0.1},
        specular: Rgb {r: 0.0, g: 0.0, b: 0.0},
        shininess: 0.0,
        ..Material::default()
    });
    let blue = Arc::new(Material {
        diffuse: Rgb {r: 0.7, g: 0.6, b: 1.0},
        specular: Rgb {r: 0.5, g: 0.4, b: 0.8},
        shininess: 25.0,
        ..Material::default()
    });

    let plane = Arc::new(MeshData::load_obj("assets/plane.obj")?);
    let dodeca = Arc::new(MeshData::load_obj("assets/dodeca.obj")?);

    // The arc
    let arc = SceneNode::from(vec![
        SceneNode::from(Geometry::new(Cube, gold.clone()))
            .scaled((0.8, 4.0, 0.8))
            .translated((-2.0, 2.0, 0.0))
            .into(),

        SceneNode::from(Geometry::new(Cube, gold.clone()))
            .scaled((0.8, 4.0, 0.8))
            .translated((2.0, 2.0, 0.0))
            .into(),

        SceneNode::from(Geometry::new(Sphere, gold.clone()))
            .scaled((4.0, 0.6, 0.6))
            .translated((0.0, 4.0, 0.0))
            .into(),
    ]).translated((0.0, 0.0, -10.0)).rotated_y(Radians::from_degrees(60.0)).into();

    // The floor
    let floor = SceneNode::from(Geometry::new(Mesh::new(plane, Shading::Flat), grass.clone()))
        .scaled(30.0)
        .into();

    // Central "sphere"
    let poly = SceneNode::from(Geometry::new(Mesh::new(dodeca, Shading::Flat), blue.clone()))
        .translated((-2.0, 1.618034, 0.0))
        .into();

    let scene = HierScene {
        root: SceneNode::from(vec![arc, floor, poly])
            .rotated_x(Radians::from_degrees(23.0))
            .translated((6.0, -2.0, -15.0))
            .into(),
        lights: vec![
            // l1
            Light {
                position: Vec3 {x: 200.0, y: 200.0, z: 400.0},
                color: Rgb {r: 0.8, g: 0.8, b: 0.8},
                ..Light::default()
            },
            // l2
            Light {
                position: Vec3 {x: 0.0, y: 5.0, z: -20.0},
                color: Rgb {r: 0.4, g: 0.4, b: 0.8},
                ..Light::default()
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

    let mut image = Image::new("hier.png", 256, 256)?;

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save()?)
}
