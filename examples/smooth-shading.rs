//! Demonstrates the smooth (Phong) shading feature

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Mesh, MeshData, Shading, Cube},
    material::Material,
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let mat_mirror = Arc::new(Material {
        diffuse: Rgb {r: 0.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.6, g: 0.6, b: 0.6},
        shininess: 1000.0,
        reflectivity: 1.0,
        ..Material::default()
    });
    let mat_wood = Arc::new(Material {
        diffuse: Rgb {r: 0.545, g: 0.353, b: 0.169},
        specular: Rgb {r: 0.5, g: 0.7, b: 0.5},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_monkey = Arc::new(Material {
        diffuse: Rgb {r: 0.68, g: 0.784, b: 0.633},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let monkey = Arc::new(MeshData::load_obj("assets/monkey.obj")?);

    let mirror = Arc::new(SceneNode::from(Geometry::new(Cube, mat_wood))
        .scaled((9.0, 0.5, 6.0))
        .rotated_x(Radians::from_degrees(10.0))
        .with_child(
            SceneNode::from(Geometry::new(Cube, mat_mirror))
                .scaled((8.1/9.0, 0.05/0.5, 5.4/6.0))
                .translated((0.0, 0.27/0.5, 0.0))
        ));

    // Make sure phong works well with hierarchical modelling
    let flat_monkey = Arc::new(SceneNode::from(Geometry::new(Mesh::new(monkey.clone(), Shading::Flat), mat_monkey.clone()))
        .rotated_y(Radians::from_degrees(10.0)));

    let smooth_monkey = Arc::new(SceneNode::from(Geometry::new(Mesh::new(monkey.clone(), Shading::Smooth), mat_monkey.clone()))
        .rotated_y(Radians::from_degrees(-10.0)));

    let scene = HierScene {
        root: SceneNode::from(vec![
            mirror,

            SceneNode::from(flat_monkey)
                .translated((-2.0, 2.0, 0.0))
                .into(),

            SceneNode::from(smooth_monkey)
                .translated((2.0, 2.0, 0.0))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 0.0, y: 5.0, z: 4.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
            Light {
                position: Vec3 {x: 0.0, y: 1.0, z: -4.0},
                color: Rgb {r: 0.5, g: 0.5, b: 0.5},
                ..Light::default()
            },
        ],
        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 10.15667, 11.579666).into(),
        center: (0.0, -5.913023, -7.571445).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(25.0),
    };

    let mut image = RgbImage::new(910, 512);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save("smooth-shading.png")?)
}
