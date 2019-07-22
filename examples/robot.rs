//! Title: "Robot Alarm Clock"
//! Author: Sunjay Varma

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, Plane, MeshData, Shading},
    kdtree::KDMesh,
    material::Material,
    texture::{Texture, ImageTexture, NormalMap},
    light::{Light, Parallelogram},
    render::Image,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Mat3, Rgb, Uv},
};

fn main() -> Result<(), Box<dyn Error>> {
    let scene = HierScene {
        root: SceneNode::from(vec![
            room()?.into(),
        ]).into(),

        lights: vec![
            // Overhead light
            Light {
                position: Vec3 {x: -2.0, y: 15.0, z: 5.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                area: Parallelogram {
                    a: Vec3 {x: 5.0, y: 0.0, z: 0.0},
                    b: Vec3 {x: 0.0, y: 0.0, z: 5.0},
                },
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (1.914036, 3.826548, 20.213762).into(),
        center: (-3.201259, 4.146196, -14.407373).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(23.0),
    };

    // let mut image = Image::new("robot.png", 1920, 1080)?;
    let mut image = Image::new("robot.png", 533, 300)?;

    // image.slice_mut((254, 44), (408, 190)).render::<RenderProgress, _>(&scene, cam,
    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.529, g: 0.808, b: 0.922} * (1.0 - uv.v) + Rgb {r: 0.086, g: 0.38, b: 0.745} * uv.v);

    Ok(image.save()?)
}

fn room() -> Result<SceneNode, Box<dyn Error>> {
    let wallpaper = Arc::new(Texture::from(ImageTexture::open("assets/wallpaper.jpg")?));
    let mat_wall = Arc::new(Material {
        // diffuse comes from texture
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        texture: Some(wallpaper),
        uv_trans: Mat3::scaling_3d(3.0),
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
            .scaled(20.0)
            .rotated_x(Radians::from_degrees(90.0))
            .translated((-2.0, 8.0, -5.0))
            .into(),

        SceneNode::from(Geometry::new(Cube, mat_table.clone()))
            .scaled((20.0, 1.0, 10.0))
            .translated((-2.0, 0.0, 0.0))
            .into(),
    ]))
}
