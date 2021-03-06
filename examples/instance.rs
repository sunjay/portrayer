//! Test for "instancing" (having multiple of the same node in different parts of the hierarchy)

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
    let stone = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.7, b: 0.7},
        specular: Rgb {r: 0.0, g: 0.0, b: 0.0},
        shininess: 0.0,
        ..Material::default()
    });
    let grass = Arc::new(Material {
        diffuse: Rgb {r: 0.1, g: 0.7, b: 0.1},
        specular: Rgb {r: 0.0, g: 0.0, b: 0.0},
        shininess: 0.0,
        ..Material::default()
    });

    let plane = Arc::new(MeshData::load_obj("assets/plane.obj")?);

    // The arc
    let arc = Arc::new(SceneNode::from(vec![
        SceneNode::from(Geometry::new(Cube, stone.clone()))
            .scaled((0.8, 4.0, 0.8))
            .translated((-2.0, 2.0, 0.0))
            .into(),

        SceneNode::from(Geometry::new(Cube, stone.clone()))
            .scaled((0.8, 4.0, 0.8))
            .translated((2.0, 2.0, 0.0))
            .into(),

        SceneNode::from(Geometry::new(Sphere, stone.clone()))
            .scaled((4.0, 0.6, 0.6))
            .translated((0.0, 4.0, 0.0))
            .into(),
    ]).translated((0.0, 0.0, -10.0)));

    // Instancing
    let mut nodes: Vec<Arc<_>> = (1..=6).map(|i| {
        SceneNode::from(arc.clone())
            .rotated_y(Radians::from_degrees(60.0 * i as f64))
            .into()
    }).collect();

    // The floor
    let floor = SceneNode::from(Geometry::new(Mesh::new(plane, Shading::Flat), grass.clone()))
        .scaled(30.0)
        .into();
    nodes.push(floor);

    // Central sphere
    let sphere = SceneNode::from(Geometry::new(Sphere, stone.clone()))
        .scaled(2.5)
        .into();
    nodes.push(sphere);

    let scene = HierScene {
        root: SceneNode::from(nodes)
            .rotated_x(Radians::from_degrees(23.0))
            .into(),
        lights: vec![
            Light {
                position: Vec3 {x: 200.0, y: 202.0, z: 430.0},
                color: Rgb {r: 0.8, g: 0.8, b: 0.8},
                ..Light::default()
            },
        ],
        ambient: Rgb {r: 0.4, g: 0.4, b: 0.4},
    };

    let cam = CameraSettings {
        eye: (0.0, 2.0, 30.0).into(),
        center: (0.0, 2.0, 29.0).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(50.0),
    };

    let mut image = Image::new("instance.png", 256, 256)?;

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save()?)
}
