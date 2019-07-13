//! Title: "Monkeys trying to make better monkeys"
//! Author: Sunjay Varma

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, Plane, Cylinder, Sphere, Mesh, MeshData, Shading},
    material::Material,
    texture::{Texture, ImageTexture, NormalMap},
    light::{Light, Parallelogram},
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let monkey_mesh = Arc::new(MeshData::load_obj("assets/monkey.obj")?);

    let scene = HierScene {
        root: SceneNode::from(vec![
            room().into(),
            desk()?.into(),
            computer(&monkey_mesh)?.into(),
        ]).into(),

        lights: vec![
            // Overhead light
            Light {
                position: Vec3 {x: 0.0, y: 13.0, z: 1.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                area: Parallelogram {
                    a: Vec3 {x: 4.0, y: 0.0, z: 0.0},
                    b: Vec3 {x: 0.0, y: 0.0, z: 4.0},
                },
                ..Light::default()
            },

            // Window
            Light {
                position: Vec3 {x: 8.0, y: 8.0, z: 8.0},
                color: Rgb {r: 0.4, g: 0.4, b: 0.4},
                area: Parallelogram {
                    a: Vec3 {x: 0.0, y: 0.0, z: 2.5},
                    b: Vec3 {x: 0.0, y: 2.5, z: 0.0},
                },
                ..Light::default()
            },
        ],
        ambient: Rgb {r: 0.3, g: 0.3, b: 0.3},
    };

    let cam = CameraSettings {
        eye: (10.626843, 11.525522, 15.875655).into(),
        center: (-11.287256, 4.506533, -10.496798).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(23.0),
    };

    let mut image = RgbImage::new(1920, 1080);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save("monkeys-making-monkeys.png")?)
}

fn room() -> SceneNode {
    let mat_floor = Arc::new(Material {
        diffuse: Rgb {r: 0.655758, g: 0.8, b: 0.753899},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_walls = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.680366, b: 0.555109},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.8},
        shininess: 25.0,
        ..Material::default()
    });
    let mat_poster = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.329194, b: 0.120657},
        specular: Rgb {r: 0.8, g: 0.8, b: 0.8},
        shininess: 25.0,
        ..Material::default()
    });

    SceneNode::from(vec![
        // Ground
        SceneNode::from(Geometry::new(Plane, mat_floor.clone()))
            .scaled(16.0)
            .translated((0.0, 0.0, 3.708507))
            .into(),

        // Left wall
        SceneNode::from(Geometry::new(Plane, mat_walls.clone()))
            .scaled(16.0)
            .rotated_z(Radians::from_degrees(-90.0))
            .translated((-6.340487, 5.0, 4.199467))
            .into(),

        // Right wall
        SceneNode::from(Geometry::new(Plane, mat_walls.clone()))
            .scaled(16.0)
            .rotated_x(Radians::from_degrees(90.0))
            .translated((0.0, 5.0, -3.2))
            .into(),

        // Poster
        SceneNode::from(Geometry::new(Plane, mat_poster.clone()))
            .scaled(4.74905)
            .rotated_z(Radians::from_degrees(-90.0))
            .translated((-6.118618, 8.043096, 3.401992))
            .into(),
    ])
}

fn desk() -> Result<SceneNode, Box<dyn Error>> {
    let wood = Arc::new(Texture::from(ImageTexture::open("assets/Wood_018_basecolor_cubemap.jpg")?));
    let wood_normals = Arc::new(NormalMap::open("assets/Wood_018_normal_cubemap.jpg")?);
    let mat_desk = Arc::new(Material {
        // diffuse comes from texture
        specular: Rgb {r: 0.5, g: 0.5, b: 0.5},
        shininess: 100.0,
        reflectivity: 0.2,
        glossy_side_length: 2.0,
        texture: Some(wood),
        normals: Some(wood_normals),
        ..Material::default()
    });

    let mut nodes = Vec::new();

    // Table top
    nodes.push(
        SceneNode::from(Geometry::new(Cube, mat_desk.clone()))
            .scaled((8.0, 0.5, 6.0))
            .translated((0.0, 5.0, 0.0))
            .into()
    );

    // Table legs
    for &x in &[-3.5, 3.5] {
        for &z in &[-2.517656, 2.517656] {
            let y = 2.54158;

            nodes.push(
                SceneNode::from(Geometry::new(Cube, mat_desk.clone()))
                    .scaled((0.470548, 4.8, 0.470548))
                    .translated(Vec3 {x, y, z})
                    .into()
            );
        }
    }

    Ok(SceneNode::from(nodes))
}

fn computer(monkey_mesh: &Arc<MeshData>) -> Result<SceneNode, Box<dyn Error>> {
    let mat_computer = Arc::new(Material {
        diffuse: Rgb {r: 0.043232, g: 0.043232, b: 0.043232},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 10.0,
        ..Material::default()
    });
    let mat_screen = Arc::new(Material {
        diffuse: Rgb {r: 0.655925, g: 0.655925, b: 0.655925},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 10.0,
        ..Material::default()
    });
    let mat_screen_text = Arc::new(Material {
        diffuse: Rgb {r: 0.8, g: 0.8, b: 0.8},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 10.0,
        ..Material::default()
    });
    let mat_hologram = Arc::new(Material {
        diffuse: Rgb {r: 0.479036, g: 0.8, b: 0.518124},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let computer_screen_base_mesh = Arc::new(MeshData::load_obj("assets/computer_screen_base.obj")?);
    let computer_edge_display_mesh = Arc::new(MeshData::load_obj("assets/computer_edge_display.obj")?);
    let screen_text_mesh = Arc::new(MeshData::load_obj("assets/text_monkey.3d.obj")?);

    Ok(SceneNode::from(vec![
        // CPU
        //TODO: Texture Map?
        SceneNode::from(Geometry::new(Cube, mat_computer.clone()))
            .scaled((1.6, 3.0, 2.0))
            .translated((-3.0, 6.74, 0.0))
            .into(),

        // Mouse
        SceneNode::from(Geometry::new(Sphere, mat_computer.clone()))
            .scaled((0.28, 0.12, 0.4))
            .translated((1.411292, 5.327119, 1.857835))
            .into(),

        // Computer screen
        SceneNode::from(Geometry::new(Mesh::new(computer_screen_base_mesh, Shading::Smooth), mat_computer.clone()))
            .translated((0.0, 5.25, 0.0))
            .into(),
        SceneNode::from(Geometry::new(Mesh::new(computer_edge_display_mesh, Shading::Flat), mat_screen.clone()))
            .translated((0.0, 7.256888, 0.0))
            .into(),
        SceneNode::from(Geometry::new(Mesh::new(screen_text_mesh, Shading::Flat), mat_screen_text.clone()))
            .translated((0.0, 9.081371, 0.01))
            .into(),

        // Holographic monkey
        SceneNode::from(Geometry::new(Mesh::new(monkey_mesh.clone(), Shading::Flat), mat_hologram.clone()))
            .scaled(1.5)
            .rotated_xzy((Radians::from_degrees(-33.2668), Radians::from_degrees(8.17821), Radians::from_degrees(-8.17821)))
            .translated((0.0, 7.0, 0.0))
            .into(),
    ]))
}
