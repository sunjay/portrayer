//! Demonstrates the normal mapping feature on all supported primitives
//!
//! Shows all the same objects from two different light angles to show how the shadows change

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Primitive, Cube, FinitePlane, Sphere},
    material::Material,
    texture::{Texture, ImageTexture},
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let tex_map = Arc::new(Texture::from(ImageTexture::open("assets/Metal_Tiles_001_basecolor.jpg")?));
    let norm_map = Arc::new(Texture::from(ImageTexture::open("assets/Metal_Tiles_001_normal.jpg")?));

    let mat_tex = Arc::new(Material {
        diffuse: Rgb {r: 0.37168, g: 0.236767, b: 0.692066},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        texture: Some(tex_map.clone()),
        ..Material::default()
    });
    let mat_tex_norm = Arc::new(Material {
        diffuse: Rgb {r: 0.37168, g: 0.236767, b: 0.692066},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        texture: Some(tex_map.clone()),
        normals: Some(norm_map),
        ..Material::default()
    });

    let tex_map_cube = Arc::new(Texture::from(ImageTexture::open("assets/Metal_Tiles_001_basecolor_cubemap.png")?));
    let norm_map_cube = Arc::new(Texture::from(ImageTexture::open("assets/Metal_Tiles_001_normal_cubemap.png")?));

    let mat_tex_cube = Arc::new(Material {
        diffuse: Rgb {r: 0.37168, g: 0.236767, b: 0.692066},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        texture: Some(tex_map_cube.clone()),
        ..Material::default()
    });
    let mat_tex_norm_cube = Arc::new(Material {
        diffuse: Rgb {r: 0.37168, g: 0.236767, b: 0.692066},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        texture: Some(tex_map_cube.clone()),
        normals: Some(norm_map_cube),
        ..Material::default()
    });

    let mat_wall_floor = Arc::new(Material {
        diffuse: Rgb {r: 0.424858, g: 0.531206, b: 0.8},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let mut nodes = Vec::new();

    // Walls + Floor
    nodes.push(
        SceneNode::from(Geometry::new(FinitePlane, mat_wall_floor.clone()))
            .scaled(30.0)
            .into()
    );
    nodes.push(
        SceneNode::from(Geometry::new(Cube, mat_wall_floor.clone()))
            .scaled((0.1, 20.0, 20.0))
            .translated((0.0, 8.0, 0.0))
            .into()
    );
    nodes.push(
        SceneNode::from(Geometry::new(Cube, mat_wall_floor.clone()))
            .scaled((30.0, 30.0, 0.4))
            .translated((0.0, 8.0, -10.0))
            .into()
    );

    // Each primitive in a column and its scale, rotation, and height
    let from_degrees = Radians::from_degrees;
    let no_scale = Vec3::one();
    let zero_rot = vek::Vec3 {x: from_degrees(0.0), y: from_degrees(0.0), z: from_degrees(0.0)};
    let prim_scale_rotation_height: &[(Primitive, _, _, _)] = &[
        (FinitePlane.into(), Vec3 {x: 2.2, y: 1.0, z: 2.0}, vek::Vec3 {x: from_degrees(90.0), ..zero_rot}, 10.9),
        (Sphere.into(), no_scale, zero_rot, 7.9),
        (Cube.into(), Vec3::from(2.0), zero_rot, 4.9),
        (Cube.into(), Vec3::from(2.0), vek::Vec3 {y: from_degrees(180.0), ..zero_rot}, 1.9),
    ];

    // The x coordinate of the center of each column and the material to use for that column
    let col_x_mat_mat_cube = &[
        (-4.0, mat_tex.clone(), mat_tex_cube.clone()),
        (-1.5, mat_tex_norm.clone(), mat_tex_norm_cube.clone()),
        (1.5, mat_tex_norm.clone(), mat_tex_norm_cube.clone()),
        (4.0, mat_tex.clone(), mat_tex_cube.clone()),
    ];

    for &(x, ref mat, ref mat_cube) in col_x_mat_mat_cube {
        for &(ref prim, scale, rot, height) in prim_scale_rotation_height {
            let mat = match prim {
                Primitive::Cube(_) => mat_cube.clone(),
                _ => mat.clone(),
            };

            nodes.push(
                SceneNode::from(Geometry::new(prim.clone(), mat))
                    .scaled(scale)
                    .rotated_xzy(rot)
                    .translated((x, height, 0.0))
                    .into()
            );
        }
    }

    let scene = HierScene {
        root: SceneNode::from(nodes).into(),

        lights: vec![
            // Left Point Light
            Light {
                position: Vec3 {x: -10.0, y: 9.0, z: 7.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },

            // Right Point Light
            Light {
                position: Vec3 {x: 10.0, y: 9.0, z: 7.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.1, g: 0.1, b: 0.1},
    };

    let cam = CameraSettings {
        eye: (0.0, 6.04746, 17.688135).into(),
        center: (0.0, 6.047459, -23.311874).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(40.0),
    };

    let mut image = RgbImage::new(910, 1024);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save("normal-mapping.png")?)
}
