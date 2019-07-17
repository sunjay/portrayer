//! Title: "The Temple of Computer Graphics"
//! Author: Sunjay Varma

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, Sphere, Cylinder, Mesh, MeshData, Shading},
    kdtree::KDMesh,
    material::{Material, WATER_REFRACTION_INDEX},
    light::Light,
    render::Image,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Rgb, Uv},
};

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

            hills()?.into(),
            lake()?.into(),

            temple_floor_1().into(),
            temple_floor_2().into(),
            temple_floor_3()?.into(),
            temple_floor_4()?.into(),
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

    let mut image = Image::new("graphics-temple.png", 1920, 1080)?;

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.529, g: 0.808, b: 0.922} * (1.0 - uv.v) + Rgb {r: 0.086, g: 0.38, b: 0.745} * uv.v);

    Ok(image.save()?)
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
        reflectivity: 0.9,
        glossy_side_length: 1.0,
        refraction_index: WATER_REFRACTION_INDEX,
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
    SceneNode::from(vec![])
}

fn temple_floor_2() -> SceneNode {
    // Generate a layout with equally spaced sections of a given width. Each section has a column
    // on each side
    let floor_width = 168.0;
    let floor_height = 20.0;
    let floor_length = 32.0;
    let floor_y_offset = 20.0;
    let floor_front_z = floor_length / 2.0;

    let sections = 4;
    let section_width = 30.0;

    let column_scale = 2.0;
    // The diameter in this case is width == length since the column has cubes at its ends
    let column_diameter = 3.2 * column_scale;
    let column_height = 8.6 * column_scale;

    // Compute the amount of space between each section.
    // -1 because there is only spacing *between* the sections, not at the end
    let section_spacing = (floor_width - column_diameter - sections as f64 * section_width) / (sections-1) as f64;

    let mut nodes = Vec::new();

    // Generate columns to hold up the ceiling
    let mat_column = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let column = Arc::new(cylinder_column(mat_column.clone()));
    for i in 0..sections * 2 {
        // Add section width on odd i
        let x = section_width * ((i+1)/2) as f64
              // Add section spacing on even i
              + section_spacing * (i/2) as f64
              // Center in the image and column size
              - floor_width / 2.0 + column_diameter / 2.0;
        // Front column
        nodes.push(
            SceneNode::from(column.clone())
                .scaled(column_scale)
                .translated((x, floor_y_offset, floor_front_z - column_diameter / 2.0))
                .into()
        );
        // Back column
        nodes.push(
            SceneNode::from(column.clone())
                .scaled(column_scale)
                .translated((x, floor_y_offset, -(floor_front_z - column_diameter / 2.0)))
                .into()
        );
    }

    // The ceiling
    let ceiling_height = floor_height - column_height;
    nodes.push(
        SceneNode::from(Geometry::new(Cube, mat_column.clone()))
            .scaled((floor_width, ceiling_height, floor_length))
            .translated((0.0, floor_y_offset + column_height + ceiling_height / 2.0, 0.0))
            .into()
    );

    // Each section contains an "idol" or "diety" which for this floor represents a cube and the
    // three types of transformations on it
    let mat_idol = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let extent = section_width.min(column_height);
    let base_idol = Arc::new(
        SceneNode::from(Geometry::new(Cube, mat_idol.clone()))
            .scaled(extent * 0.5)
            .rotated_y(Radians::from_degrees(30.0))
    );
    let idols = vec![
        SceneNode::from(base_idol.clone()),
        SceneNode::from(base_idol.clone())
            .scaled((1.0, 0.4, 1.0)),
        SceneNode::from(base_idol.clone())
            .rotated_z(Radians::from_degrees(80.0)),
        SceneNode::from(vec![
            SceneNode::from(base_idol.clone())
                .scaled(0.5)
                .translated((-extent / 4.0, extent / 8.0, -floor_length / 8.0))
                .into(),
            SceneNode::from(base_idol.clone())
                .scaled(0.5)
                .translated((extent / 4.0, -extent / 8.0, floor_length / 8.0))
                .into(),
        ]),
    ];
    assert_eq!(idols.len(), sections);

    for (i, idol) in idols.into_iter().enumerate() {
        let x = section_width * (i + 1) as f64 + section_spacing * i as f64
              // Center in the image and section width
              - floor_width / 2.0 - section_width / 2.0 + column_diameter / 2.0;
        nodes.push(
            idol.translated((x, floor_y_offset + column_height / 2.0, 0.0)).into()
        );
    }

    SceneNode::from(nodes)
}

fn temple_floor_3() -> Result<SceneNode, Box<dyn Error>> {
    let floor_width = 117.6;
    let floor_length = 25.6;
    let floor_height = 20.0;
    let floor_y_offset = 40.0;

    let puppet_height = 17.2;
    let puppet_y_offset = 44.083061;

    let ceiling_height = floor_height - puppet_height;
    let ceiling_y_offset = floor_y_offset + puppet_height + ceiling_height / 2.0;

    let mat_puppet = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let puppet_model = Arc::new(MeshData::load_obj("assets/tog_puppet.obj")?);
    let puppet = Arc::new(SceneNode::from(Geometry::new(KDMesh::new(&*puppet_model, Shading::Smooth), mat_puppet))
        .translated((0.0, puppet_y_offset, 0.0)));

    let mat_ceiling = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    Ok(SceneNode::from(vec![
        // The ceiling
        SceneNode::from(Geometry::new(Cube, mat_ceiling.clone()))
            .scaled((floor_width, ceiling_height, floor_length))
            .translated((0.0, ceiling_y_offset, 0.0))
            .into(),

        // Left puppet
        SceneNode::from(puppet.clone())
            .rotated_y(Radians::from_degrees(90.0))
            .translated((-55.1, 0.0, 0.0))
            .into(),
        // Center puppet
        SceneNode::from(puppet.clone())
            .translated((0.0, 0.0, -5.0))
            .into(),
        // Right puppet
        SceneNode::from(puppet.clone())
            .rotated_y(Radians::from_degrees(-90.0))
            .translated((55.1, 0.0, 0.0))
            .into(),
    ]))
}

fn temple_floor_4() -> Result<SceneNode, Box<dyn Error>> {
    let mat_crystal = Arc::new(Material {
        //TODO: Replace this material
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let monkey_model = Arc::new(MeshData::load_obj("assets/monkey.obj")?);
    let teapot_model = Arc::new(MeshData::load_obj("assets/teapot.obj")?);
    let cow_model = Arc::new(MeshData::load_obj("assets/cow.obj")?);

    Ok(SceneNode::from(vec![
        // Monkey
        SceneNode::from(Geometry::new(Mesh::new(monkey_model, Shading::Smooth), mat_crystal.clone()))
            .scaled(8.0)
            .rotated_xzy((Radians::from_degrees(-34.9072), Radians::from_degrees(25.0), Radians::from_degrees(0.0)))
            .translated((-30.0, 64.214905, 1.0))
            .into(),

        // Teapot
        SceneNode::from(Geometry::new(KDMesh::new(&*teapot_model, Shading::Smooth), mat_crystal.clone()))
            .scaled(0.6)
            .rotated_y(Radians::from_degrees(-55.0))
            .translated((0.0, 59.857296, 0.0))
            .into(),

        // Cow
        SceneNode::from(Geometry::new(KDMesh::new(&*cow_model, Shading::Smooth), mat_crystal.clone()))
            .scaled(1.5)
            .rotated_y(Radians::from_degrees(-125.0))
            .translated((30.0, 65.31517, 0.0))
            .into(),
    ]))
}

/// A cylinderical column with center at its bottom middle
fn cylinder_column(mat_column: Arc<Material>) -> SceneNode {
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
