//! Title: "The Temple of Computer Graphics"
//! Author: Sunjay Varma

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Sphere, Cube, Mesh, MeshData, Shading},
    material::{Material, WATER_REFRACTION_INDEX},
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb, Uv},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let mat_water = Arc::new(Material {
        diffuse: Rgb {r: 0.0, g: 0.0, b: 0.1},
        specular: Rgb {r: 0.5, g: 0.5, b: 0.5},
        shininess: 100.0,
        // reflectivity: 0.9,
        // glossy_side_length: 3.0,
        // refraction_index: WATER_REFRACTION_INDEX,
        ..Material::default()
    });

    let mat_dirt = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.579469, b: 0.342056},
        ..Material::default()
    });

    let mat_grass = Arc::new(Material {
        diffuse: Rgb {r: 0.156113, g: 0.8, b: 0.152911},
        ..Material::default()
    });

    let mat_temple_block = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.491697, b: 0.753542},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let grass_model = Arc::new(MeshData::load_obj("assets/tog_grass.obj")?);
    let underwater_land_model = Arc::new(MeshData::load_obj("assets/tog_underwater_land.obj")?);
    let water_model = Arc::new(MeshData::load_obj("assets/tog_water.obj")?);

    let scene = HierScene {
        root: SceneNode::from(vec![
            SceneNode::from(Geometry::new(Cube, mat_temple_block.clone()))
                .scaled((240.0, 20.0, 40.0))
                .translated((0.0, 10.0, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Cube, mat_temple_block.clone()))
                .scaled((168.0, 20.0, 32.0))
                .translated((0.0, 30.0, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Cube, mat_temple_block.clone()))
                .scaled((117.599998, 20.0, 25.6))
                .translated((0.0, 50.0, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Cube, mat_temple_block.clone()))
                .scaled((82.32, 20.0, 20.480001))
                .translated((0.0, 70.0, 0.0))
                .into(),

            SceneNode::from(Geometry::new(Mesh::new(grass_model, Shading::Smooth), mat_grass))
                .translated((1.958125, 16.093138, -86.113747))
                .into(),

            SceneNode::from(Geometry::new(Mesh::new(water_model, Shading::Smooth), mat_water))
                .translated((1.499644, -18.555456, 257.387817))
                .into(),

            // Flat to speed up rendering since the normals don't super matter for this (not visible)
            SceneNode::from(Geometry::new(Mesh::new(underwater_land_model, Shading::Flat), mat_dirt))
                .translated((2.110489, -35.596691, 299.865814))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 190.0, y: 98.0, z: 151.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (0.0, 61.971188, 546.971191).into(),
        center: (0.0, -13.390381, -585.524353).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(25.0),
    };

    // let mut image = RgbImage::new(1920, 1080);
    let mut image = RgbImage::new(533, 300);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.529, g: 0.808, b: 0.922} * (1.0 - uv.v) + Rgb {r: 0.086, g: 0.38, b: 0.745} * uv.v);

    Ok(image.save("graphics-temple.png")?)
}
