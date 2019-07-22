//! Demonstrates the Transmission / Refraction feature on a glass with a straw

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, Plane, Cylinder},
    material::{Material, WATER_REFRACTION_INDEX},
    texture::{Texture, ImageTexture, NormalMap},
    light::Light,
    render::Image,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};

fn main() -> Result<(), Box<dyn Error>> {
    let scene = HierScene {
        root: SceneNode::from(vec![
            room()?.into(),
            drink().translated((0.0, 0.2, 0.0)).into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 0.0, y: 27.0, z: 5.0},
                color: Rgb {r: 0.5, g: 0.5, b: 0.5},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 3.2, 7.151111).into(),
        center: (0.0, 0.091525, -5.719519).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(23.0),
    };

    let mut image = Image::new("water-glass.png", 910, 512)?;

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save()?)
}

fn room() -> Result<SceneNode, Box<dyn Error>> {
    let brick = Arc::new(Texture::from(ImageTexture::open("assets/Brick_Wall_013_COLOR.jpg")?));
    let brick_normals = Arc::new(NormalMap::open("assets/Brick_Wall_013_NORM.jpg")?);
    let mat_wall = Arc::new(Material {
        // diffuse comes from texture
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        texture: Some(brick),
        normals: Some(brick_normals),
        ..Material::default()
    });

    let wood = Arc::new(Texture::from(ImageTexture::open("assets/Wood_018_basecolor_cubemap.jpg")?));
    let wood_normals = Arc::new(NormalMap::open("assets/Wood_018_normal_cubemap.jpg")?);
    let mat_table = Arc::new(Material {
        // diffuse comes from texture
        specular: Rgb {r: 0.5, g: 0.5, b: 0.5},
        shininess: 100.0,
        reflectivity: 0.2,
        glossy_side_length: 2.0,
        texture: Some(wood),
        normals: Some(wood_normals),
        ..Material::default()
    });

    Ok(SceneNode::from(vec![
        SceneNode::from(Geometry::new(Plane, mat_wall.clone()))
            .scaled(10.0)
            .rotated_x(Radians::from_degrees(90.0))
            .translated((0.0, 1.0, -2.0))
            .into(),

        SceneNode::from(Geometry::new(Cube, mat_table.clone()))
            .scaled((8.0, 0.4, 4.0))
            .translated((0.0, 0.0, -0.2))
            .into(),
    ]))
}

fn drink() -> SceneNode {
    let mat_water = Arc::new(Material {
        diffuse: Rgb {r: 0.0, g: 0.0, b: 0.1},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        reflectivity: 0.9,
        refraction_index: WATER_REFRACTION_INDEX,
        ..Material::default()
    });

    let mat_straw = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    SceneNode::from(vec![
        SceneNode::from(Geometry::new(Cylinder, mat_water.clone()))
            .scaled((1.0, 1.4, 1.0))
            .translated((0.0, 0.7, 0.0))
            .into(),

        SceneNode::from(Geometry::new(Cylinder, mat_straw.clone()))
            .scaled((0.1, 2.0, 0.1))
            .rotated_z(Radians::from_degrees(28.4282))
            .translated((-0.165556, 0.911109, 0.1))
            .into(),
    ])
}
