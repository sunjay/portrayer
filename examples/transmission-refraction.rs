//! Demonstrates the Transmission / Refraction feature

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, Sphere, Plane, Mesh, MeshData, Shading},
    material::{Material, WATER_REFRACTION_INDEX, WINDOW_GLASS_REFRACTION_INDEX},
    texture::{Texture, ImageTexture, NormalMap},
    light::{Light, Parallelogram},
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let mat_glass = Arc::new(Material {
        diffuse: Rgb {r: 0.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        reflectivity: 1.0,
        refraction_index: WINDOW_GLASS_REFRACTION_INDEX,
        ..Material::default()
    });
    let wood = Arc::new(Texture::from(ImageTexture::open("assets/Wood_018_basecolor_cubemap.jpg")?));
    let wood_normals = Arc::new(NormalMap::open("assets/Wood_018_normal_cubemap.jpg")?);
    let mat_table = Arc::new(Material {
        // diffuse comes from texture
        specular: Rgb {r: 0.5, g: 0.5, b: 0.5},
        shininess: 100.0,
        reflectivity: 0.3,
        glossy_side_length: 2.0,
        texture: Some(wood),
        normals: Some(wood_normals),
        ..Material::default()
    });

    let mat_red = Arc::new(Material {
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let scene = HierScene {
        root: SceneNode::from(vec![
            SceneNode::from(Geometry::new(Cube, mat_table.clone()))
                .scaled((20.0, 1.0, 10.0))
                .translated((0.0, -0.5, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Cube, mat_glass.clone()))
                .scaled(2.0)
                .translated((0.0, 1.0, 1.0))
                .into(),

            SceneNode::from(Geometry::new(Sphere, mat_red.clone()))
                .translated((0.0, 1.0, -1.0))
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
                    a: Vec3 {x: 0.0, y: 0.5, z: 0.0},
                    b: Vec3 {x: 0.5, y: 0.0, z: 0.0},
                },
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 6.0, 22.0).into(),
        center: (0.0, -4.206809, -12.868293).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(25.0),
    };

    let mut image = RgbImage::new(910, 512);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save("transmission-refraction.png")?)
}
