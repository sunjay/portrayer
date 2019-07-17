//! Title: "The Temple of Computer Graphics"
//! Author: Sunjay Varma

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, Sphere, Cylinder, MeshData, Shading},
    kdtree::KDMesh,
    material::{Material, WATER_REFRACTION_INDEX},
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb, Uv},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let mat_temple_block = Arc::new(Material {
        diffuse: Rgb {r: 0.913099, g: 0.913099, b: 0.715694},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let scene = HierScene {
        root: SceneNode::from(vec![
            //TODO: All temple blocks should be removed by the time we're done
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

            hills()?.into(),
            lake()?.into(),

            temple_floor_1().into(),
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

fn hills() -> Result<SceneNode, Box<dyn Error>> {
    let mat_grass = Arc::new(Material {
        diffuse: Rgb {r: 0.376, g: 0.502, b: 0.22},
        ..Material::default()
    });

    let grass_model = Arc::new(MeshData::load_obj("assets/tog_grass.obj")?);

    Ok(SceneNode::from(Geometry::new(KDMesh::new(&*grass_model, Shading::Smooth), mat_grass))
        .translated((1.958125, 16.093138, -86.113747)))
}

fn lake() -> Result<SceneNode, Box<dyn Error>> {
    let mat_water = Arc::new(Material {
        diffuse: Rgb {r: 0.0, g: 0.0, b: 0.1},
        specular: Rgb {r: 0.5, g: 0.5, b: 0.5},
        shininess: 100.0,
        // reflectivity: 0.9,
        // glossy_side_length: 1.0,
        // refraction_index: WATER_REFRACTION_INDEX,
        ..Material::default()
    });

    let mat_dirt = Arc::new(Material {
        // Color of algae makes the water blue!
        diffuse: Rgb {r: 0.592, g: 0.671, b: 0.055},
        ..Material::default()
    });

    let underwater_land_model = Arc::new(MeshData::load_obj("assets/tog_underwater_land.obj")?);

    Ok(SceneNode::from(vec![
        SceneNode::from(Geometry::new(Cube, mat_water))
            .scaled((600.0, 200.0, 600.0))
            .translated((0.0, -107.0, 300.0))
            .into(),

        // Flat shaded to speed up rendering since the normals don't super matter for this (not visible)
        SceneNode::from(Geometry::new(KDMesh::new(&*underwater_land_model, Shading::Flat), mat_dirt))
            .translated((0.0, -107.0, 300.0))
            .into(),
    ]))
}

fn temple_floor_1() -> SceneNode {
    // Generate a layout with equally spaced sections of a given width. Each section has a column
    // on each side
    let floor_width = 240.0;
    let floor_height = 20.0;
    let floor_y_offset = 0.0;
    let floor_front_z = 20.0;

    let sections = 4;
    let section_width = 40.0;

    let column_width = 3.2;
    let column_height = 8.6;

    // -1 because there is only spacing *between* the sections, not at the end
    let section_spacing = (floor_width - sections as f64 * section_width) / (sections-1) as f64;
    let column_scale = floor_height / column_height;

    let mut nodes = Vec::new();

    let column = Arc::new(cylinder_column());
    for i in 0..sections * 2 {
        // Add section width on odd i
        let x = section_width * ((i+1)/2) as f64
              // Add section spacing on even i
              + section_spacing * (i/2) as f64
              // Center in the image and column width
              - floor_width / 2.0 + column_width / 2.0;
        nodes.push(
            SceneNode::from(column.clone())
                .scaled(column_scale)
                .translated((x, floor_y_offset, floor_front_z))
                .into()
        );
    }

    SceneNode::from(nodes)
}

/// A cylinderical column with center at its bottom middle
fn cylinder_column() -> SceneNode {
    let mat_column = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    SceneNode::from(vec![
        SceneNode::from(Geometry::new(Cube, mat_column.clone()))
            .scaled((3.2, 1.0, 3.2))
            .translated((0.0, 3.8, 0.0))
            .into(),
        SceneNode::from(Geometry::new(Cube, mat_column.clone()))
            .scaled((3.2, 1.0, 3.2))
            .translated((0.0, -3.8, 0.0))
            .into(),

        SceneNode::from(Geometry::new(Sphere, mat_column.clone()))
            .scaled((1.5, 0.5, 1.5))
            .translated((0.0, 3.0, 0.0))
            .into(),
        SceneNode::from(Geometry::new(Sphere, mat_column.clone()))
            .scaled((1.5, 0.5, 1.5))
            .translated((0.0, -3.0, 0.0))
            .into(),

        SceneNode::from(Geometry::new(Cylinder, mat_column.clone()))
            .scaled((2.0, 6.0, 2.0))
            .into(),
    ]).translated((0.0, 4.3, 0.0))
}
