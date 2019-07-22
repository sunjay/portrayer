//! Title: "Robot Alarm Clock"
//! Author: Sunjay Varma

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Mesh, Cube, Plane, MeshData, Shading},
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
            robot()?.into(),
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

    // let mut image = Image::new("robot_alarm_clock.png", 1920, 1080)?;
    let mut image = Image::new("robot_alarm_clock.png", 533, 300)?;

    // image.slice_mut((254, 44), (408, 190)).render::<RenderProgress, _>(&scene, cam,
    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.529, g: 0.808, b: 0.922} * (1.0 - uv.v) + Rgb {r: 0.086, g: 0.38, b: 0.745} * uv.v);

    Ok(image.save()?)
}

fn room() -> Result<SceneNode, Box<dyn Error>> {
    let wallpaper = Arc::new(Texture::from(ImageTexture::open("assets/robot_alarm_clock/wallpaper.jpg")?));
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

fn robot() -> Result<SceneNode, Box<dyn Error>> {
    Ok(SceneNode::from(vec![
        robot_base()?.into(),
    ]))
}

fn robot_base() -> Result<SceneNode, Box<dyn Error>> {
    let mat_robot_metal = Arc::new(Material {
        diffuse: Rgb {r: 0.211857, g: 0.772537, b: 0.8971},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.8},
        shininess: 100.0,
        reflectivity: 0.3,
        glossy_side_length: 0.5,
        ..Material::default()
    });

    let robot_base_model = Arc::new(MeshData::load_obj("assets/robot_alarm_clock/robot_base.obj")?);

    Ok(SceneNode::from(vec![
        SceneNode::from(Geometry::new(KDMesh::new(&robot_base_model, Shading::Smooth), mat_robot_metal.clone()))
            .translated((0.0, 1.002795, -0.209603))
            .into(),

        clock()?
            .into(),

        clock_buttons()?
            .into(),
    ]))
}

fn clock() -> Result<SceneNode, Box<dyn Error>> {
    let mat_clock_case = Arc::new(Material {
        diffuse: Rgb {r: 1.0, g: 1.0, b: 1.0},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let mat_time_bg = Arc::new(Material {
        diffuse: Rgb {r: 0.059252, g: 0.059252, b: 0.059252},
        ..Material::default()
    });

    let mat_time = Arc::new(Material {
        diffuse: Rgb {r: 1.0, g: 0.0, b: 0.0},
        ..Material::default()
    });

    let clock_case_model = Arc::new(MeshData::load_obj("assets/robot_alarm_clock/robot_base_clock_case.obj")?);
    let clock_time_model = Arc::new(MeshData::load_obj("assets/robot_alarm_clock/robot_base_clock_time.obj")?);

    let angle = -6.62911;
    Ok(SceneNode::from(vec![
        //TODO: KDMesh doesn't work for this for some reason...
        SceneNode::from(Geometry::new(Mesh::new(clock_case_model, Shading::Smooth), mat_clock_case))
            .rotated_x(Radians::from_degrees(angle))
            .translated((0.0, 1.228179, 0.350087))
            .into(),

        SceneNode::from(Geometry::new(Plane, mat_time_bg))
            .scaled((2.966855, 1.0, 0.684205))
            .rotated_x(Radians::from_degrees(90.0 + angle))
            .translated((0.0, 1.294323, 0.919223))
            .into(),

        //TODO: KDMesh doesn't work for this for some reason...
        SceneNode::from(Geometry::new(Mesh::new(clock_time_model, Shading::Flat), mat_time))
            .rotated_x(Radians::from_degrees(83.2518 - 90.0))
            .translated((0.0, 1.535768, 0.921095))
            .into(),
    ]))
}

fn clock_buttons() -> Result<SceneNode, Box<dyn Error>> {
    let mat_clock_button = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.103095, b: 0.086502},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let x_values = &[-1.2, -0.4, 0.4, 1.2];

    let clock_button_model = Arc::new(MeshData::load_obj("assets/robot_alarm_clock/robot_base_clock_button.obj")?);
    //TODO: KDMesh doesn't work for this for some reason...
    let clock_button = Arc::new(SceneNode::from(Geometry::new(Mesh::new(clock_button_model, Shading::Smooth), mat_clock_button)));

    let mut nodes = Vec::new();

    for &x in x_values {
        nodes.push(
            SceneNode::from(clock_button.clone())
                .rotated_x(Radians::from_degrees(15.0))
                .translated((x, 1.7, -0.2))
                .into()
        );
    }

    Ok(SceneNode::from(nodes))
}
