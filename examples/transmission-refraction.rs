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
        texture: Some(wood),
        normals: Some(wood_normals),
        ..Material::default()
    });

    let scene = HierScene {
        root: SceneNode::from(vec![
            SceneNode::from(Geometry::new(Cube, mat_table.clone()))
                .scaled((20.0, 1.0, 2.5))
                .translated((0.0, 0.0, 1.3))
                .into(),

            // Front glass
            SceneNode::from(Geometry::new(Cube, mat_glass.clone()))
                .scaled((20.0, 10.0, 0.2))
                .translated((0.0, 5.0, 0.0))
                .into(),

            room().into(),
            tank()?.into(),
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
        eye: (0.0, 18.605604, 21.199203).into(),
        center: (0.0, -4.20681, -12.868294).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(23.0),
    };

    let mut image = RgbImage::new(910, 512);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save("transmission-refraction.png")?)
}

fn room() -> SceneNode {
    let mat_walls = Arc::new(Material {
        diffuse: Rgb {r: 0.607917, g: 0.8, b: 0.551884},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    SceneNode::from(vec![
        // Back wall
        SceneNode::from(Geometry::new(Plane, mat_walls.clone()))
            .scaled((20.0, 1.0, 20.0))
            .rotated_x(Radians::from_degrees(90.0))
            .translated((0.0, 3.0, -10.0))
            .into(),

        // Right wall
        SceneNode::from(Geometry::new(Plane, mat_walls.clone()))
            .scaled((20.0, 1.0, 12.0))
            .rotated_z(Radians::from_degrees(90.0))
            .translated((10.0, 3.0, -4.0))
            .into(),

        // Left wall
        SceneNode::from(Geometry::new(Plane, mat_walls.clone()))
            .scaled((20.0, 1.0, 12.0))
            .rotated_z(Radians::from_degrees(-90.0))
            .translated((-10.0, 3.0, -4.0))
            .into(),
    ])
}

fn tank() -> Result<SceneNode, Box<dyn Error>> {
    let tiles = Arc::new(Texture::from(ImageTexture::open("assets/Tiles_017_basecolor_cubemap.jpg")?));
    let tiles_normals = Arc::new(NormalMap::open("assets/Tiles_017_normal_cubemap.jpg")?);

    let mat_tank = Arc::new(Material {
        // diffuse comes from texture
        specular: Rgb {r: 0.5, g: 0.5, b: 0.5},
        shininess: 100.0,
        texture: Some(tiles),
        normals: Some(tiles_normals),
        ..Material::default()
    });

    let mut nodes = Vec::new();

    // Using loops and lots of cubes to preserve aspect ratio of texture

    // Add front and back of tank
    for i in 0..4 {
        nodes.push(
            SceneNode::from(Geometry::new(Cube, mat_tank.clone()))
                .scaled((5.0, 5.0, 0.2))
                .translated((i as f64 * 5.0 - 7.5, -2.0, -10.0))
                .into(),
        );
        nodes.push(
            SceneNode::from(Geometry::new(Cube, mat_tank.clone()))
                .scaled((5.0, 5.0, 0.2))
                .translated((i as f64 * 5.0 - 7.5, -2.0, 0.0))
                .into(),
        );
    }

    // Add side walls
    for i in 0..2 {
        nodes.push(
            SceneNode::from(Geometry::new(Cube, mat_tank.clone()))
                .scaled((0.2, 5.0, 5.0))
                .translated((-10.0, -2.0, -(i as f64 * 5.0 + 2.5)))
                .into(),
        );
        nodes.push(
            SceneNode::from(Geometry::new(Cube, mat_tank.clone()))
                .scaled((0.2, 5.0, 5.0))
                .translated((10.0, -2.0, -(i as f64 * 5.0 + 2.5)))
                .into(),
        );
    }

    // Add bottom
    for x in 0..4 {
        for y in 0..2 {
            nodes.push(
                SceneNode::from(Geometry::new(Cube, mat_tank.clone()))
                    .scaled((5.0, 0.2, 5.0))
                    .translated((x as f64 * 5.0 - 7.5, -4.0, -(y as f64 * 5.0 + 2.5)))
                    .into(),
            );
        }
    }

    Ok(SceneNode::from(nodes))
}
