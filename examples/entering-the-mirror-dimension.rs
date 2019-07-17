//! Title: "Entering The Mirror Dimension"
//! Author: Sunjay Varma

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
    // Materials

    let mat_mirror_frame = Arc::new(Material {
        diffuse: Rgb {r: 0.29, g: 0.204, b: 0.145},
        specular: Rgb {r: 0.0, g: 0.0, b: 0.0},
        shininess: 1.0,
        ..Material::default()
    });
    let mat_mirror = Arc::new(Material {
        diffuse: Rgb {r: 0.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.8},
        shininess: 1000.0,
        reflectivity: 1.0,
        ..Material::default()
    });
    let mat_floor = Arc::new(Material {
        diffuse: Rgb {r: 0.016, g: 0.384, b: 0.0},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.8},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_body = Arc::new(Material {
        diffuse: Rgb {r: 0.906, g: 0.22, b: 0.282},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.8},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_head = Arc::new(Material {
        diffuse: Rgb {r: 0.086, g: 0.671, b: 0.906},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.8},
        shininess: 50.0,
        ..Material::default()
    });
    let mat_eyes = Arc::new(Material {
        diffuse: Rgb {r: 0.3, g: 0.3, b: 0.3},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.8},
        shininess: 1000.0,
        reflectivity: 0.9,
        ..Material::default()
    });
    let mat_arms = Arc::new(Material {
        diffuse: Rgb {r: 0.345, g: 0.588, b: 0.906},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.8},
        shininess: 1.0,
        ..Material::default()
    });

    let monkey = Arc::new(MeshData::load_obj("assets/monkey.obj")?);
    let plane = Arc::new(MeshData::load_obj("assets/plane.obj")?);

    let mirror = SceneNode::from(vec![
        // mirror frame
        SceneNode::from(Geometry::new(Cube, mat_mirror_frame.clone()))
            .scaled((3.96, 5.5, 0.4))
            .translated((0.0, 2.75, 0.0))
            .into(),

        // mirror glass
        SceneNode::from(Geometry::new(Cube, mat_mirror.clone()))
            .scaled((3.6, 5.0, 0.1))
            .translated((0.0, 2.75, 0.2))
            .into(),
    ]).translated((0.0, 0.0, -1.3)).into();

    let monkey_character = SceneNode::from(vec![
        // torso
        SceneNode::from(Geometry::new(Cube, mat_body.clone()))
            .scaled((0.545055, 2.6, 0.545055))
            .translated((0.0, 1.3, 0.0))
            .into(),

        // head
        SceneNode::from(Geometry::new(Mesh::new(monkey, Shading::Flat), mat_head.clone()))
            .scaled((1.0, 1.0, 1.0))
            .rotated_y(Radians::from_degrees(180.0))
            .translated((0.0, 2.7, 0.0))
            .with_children(vec![
                // left eye
                SceneNode::from(Geometry::new(Sphere, mat_eyes.clone()))
                .scaled((0.1, 0.1, 0.05))
                .translated((0.35, 0.24, 0.8))
                .into(),

                // right eye
                SceneNode::from(Geometry::new(Sphere, mat_eyes.clone()))
                .scaled((0.1, 0.1, 0.05))
                .translated((-0.35, 0.24, 0.8))
                .into(),
            ])
            .into(),

        // left upper arm
        SceneNode::from(Geometry::new(Sphere, mat_arms.clone()))
            .scaled((0.2, 0.63, 0.2))
            .rotated_xzy(Vec3::from((161.156, 107.062, -133.944)).map(Radians::from_degrees))
            .translated((-0.388703, 1.715599, -0.2))
            .into(),
        // left lower arm
        SceneNode::from(Geometry::new(Sphere, mat_arms.clone()))
            .scaled((0.2, 0.56, 0.2))
            .rotated_xzy(Vec3::from((127.221, 42.0695, -104.823)).map(Radians::from_degrees))
            .translated((-0.711297, 1.284401, -1.0))
            .into(),
        // left mirror bubble
        SceneNode::from(Geometry::new(Sphere, mat_mirror.clone()))
            .scaled((0.5, 0.5, 0.3))
            .translated((-0.711297, 1.284401, -1.20))
            .into(),

        // right upper arm
        SceneNode::from(Geometry::new(Sphere, mat_arms.clone()))
            .scaled((0.2, 0.63, 0.2))
            .rotated_xzy(Vec3::from((92.3684, -57.6199, 38.2278)).map(Radians::from_degrees))
            .translated((0.581161, 1.984976, -0.2))
            .into(),
        // right lower arm
        SceneNode::from(Geometry::new(Sphere, mat_arms.clone()))
            .scaled((0.2, 0.56, 0.2))
            .rotated_xzy(Vec3::from((91.5166, -11.239, 28.419)).map(Radians::from_degrees))
            .translated((1.118839, 2.015024, -1.0))
            .into(),
        // right mirror bubble
        SceneNode::from(Geometry::new(Sphere, mat_mirror.clone()))
            .scaled((0.5, 0.5, 0.3))
            .translated((1.118839, 2.015024, -1.20))
            .into(),
    ]).into();

    // The floor
    let floor = SceneNode::from(Geometry::new(Mesh::new(plane, Shading::Flat), mat_floor.clone()))
        .scaled(20.0)
        .into();

    let scene = HierScene {
        root: SceneNode::from(vec![mirror, floor, monkey_character]).into(),
        lights: vec![
            // face_light
            Light {
                position: Vec3 {x: 2.5, y: 3.5, z: -1.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
            // white_light
            Light {
                position: Vec3 {x: 10.0, y: 10.0, z: 0.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
            // blue_light
            Light {
                position: Vec3 {x: -9.0, y: 4.0, z: 0.0},
                color: Rgb {r: 0.406471, g: 0.901283, b: 1.0},
                ..Light::default()
            },
        ],
        ambient: Rgb {r: 0.2, g: 0.2, b: 0.2},
    };

    let cam = CameraSettings {
        eye: (5.545485, 2.966984, 1.795613).into(),
        center: (-4.348584, 2.148794, -3.057839).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(30.0),
    };

    let mut image = Image::new("entering-the-mirror-dimension.png", 800, 600)?;

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save()?)
}
