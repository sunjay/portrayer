//! Demonstrates the normal mapping feature on all supported primitives
//!
//! Shows all the same objects from two different light angles to show how the shadows change

use std::error::Error;
use std::sync::Arc;

use portrayer::{
    scene::{HierScene, SceneNode, Geometry},
    primitive::{Cube, FinitePlane, Sphere},
    material::Material,
    texture::{Texture, ImageTexture, NormalMap},
    light::Light,
    render::Render,
    reporter::RenderProgress,
    camera::CameraSettings,
    math::{Radians, Vec3, Uv, Rgb},
};
use image::RgbImage;

fn main() -> Result<(), Box<dyn Error>> {
    let tex_map_plane = Arc::new(Texture::from(ImageTexture::open("assets/Terracotta_Tiles_002_Base_Color.jpg")?));
    let norm_map_plane = Arc::new(NormalMap::open("assets/Terracotta_Tiles_002_Normal.jpg")?);

    let mat_tex_plane = Arc::new(Material {
        diffuse: Rgb {r: 0.37168, g: 0.236767, b: 0.692066},
        specular: Rgb {r: 0.4, g: 0.4, b: 0.4},
        shininess: 25.0,
        texture: Some(tex_map_plane.clone()),
        ..Material::default()
    });
    let mat_tex_plane_norm = Arc::new(Material {
        diffuse: Rgb {r: 0.37168, g: 0.236767, b: 0.692066},
        specular: Rgb {r: 0.4, g: 0.4, b: 0.4},
        shininess: 25.0,
        texture: Some(tex_map_plane.clone()),
        normals: Some(norm_map_plane),
        ..Material::default()
    });

    let tex_map_sphere = Arc::new(Texture::from(ImageTexture::open("assets/Rock_033_baseColor_2.jpg")?));
    let norm_map_sphere = Arc::new(NormalMap::open("assets/Rock_033_normal_2.jpg")?);

    let mat_tex_sphere = Arc::new(Material {
        diffuse: Rgb {r: 0.37168, g: 0.236767, b: 0.692066},
        specular: Rgb {r: 0.6, g: 0.6, b: 0.6},
        shininess: 25.0,
        texture: Some(tex_map_sphere.clone()),
        ..Material::default()
    });
    let mat_tex_sphere_norm = Arc::new(Material {
        diffuse: Rgb {r: 0.37168, g: 0.236767, b: 0.692066},
        specular: Rgb {r: 0.6, g: 0.6, b: 0.6},
        shininess: 25.0,
        texture: Some(tex_map_sphere),
        normals: Some(norm_map_sphere),
        ..Material::default()
    });

    let tex_map_cube = Arc::new(Texture::from(ImageTexture::open("assets/Stone_Wall_007_COLOR_cubemap.jpg")?));
    let norm_map_cube = Arc::new(NormalMap::open("assets/Stone_Wall_007_NORM_cubemap.jpg")?);

    let mat_tex_cube = Arc::new(Material {
        diffuse: Rgb {r: 0.37168, g: 0.236767, b: 0.692066},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        texture: Some(tex_map_cube.clone()),
        ..Material::default()
    });
    let mat_tex_cube_norm = Arc::new(Material {
        diffuse: Rgb {r: 0.37168, g: 0.236767, b: 0.692066},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        texture: Some(tex_map_cube),
        normals: Some(norm_map_cube),
        ..Material::default()
    });

    let mat_wall_floor = Arc::new(Material {
        diffuse: Rgb {r: 0.424858, g: 0.531206, b: 0.8},
        specular: Rgb {r: 0.3, g: 0.3, b: 0.3},
        shininess: 25.0,
        ..Material::default()
    });

    let scene = HierScene {
        root: SceneNode::from(vec![
            // Floor
            SceneNode::from(Geometry::new(FinitePlane, mat_wall_floor))
                .scaled(40.0)
                .translated((0.0, -1.0, 0.0))
                .into(),

            // Left - Texture Only
            SceneNode::from(Geometry::new(FinitePlane, mat_tex_plane))
                .scaled(6.0)
                .rotated_x(Radians::from_degrees(90.0))
                .translated((-4.0, 2.0, -6.0))
                .into(),

            SceneNode::from(Geometry::new(Cube, mat_tex_cube.clone()))
                .scaled(2.0)
                .translated((-7.0, 0.0, -1.0))
                .into(),
            SceneNode::from(Geometry::new(Sphere, mat_tex_sphere.clone()))
                .translated((-7.0, 2.0, -1.0))
                .into(),

            SceneNode::from(Geometry::new(Cube, mat_tex_cube.clone()))
                .scaled(2.0)
                .translated((-2.0, 0.0, 3.0))
                .into(),
            SceneNode::from(Geometry::new(Sphere, mat_tex_sphere.clone()))
                .translated((-2.0, 2.0, 3.0))
                .into(),

            // Right - Normal + Texture
            SceneNode::from(Geometry::new(FinitePlane, mat_tex_plane_norm))
                .scaled(6.0)
                .rotated_x(Radians::from_degrees(90.0))
                .translated((4.0, 2.0, -6.0))
                .into(),

            SceneNode::from(Geometry::new(Cube, mat_tex_cube_norm.clone()))
                .scaled(2.0)
                .translated((7.0, 0.0, -1.0))
                .into(),
            SceneNode::from(Geometry::new(Sphere, mat_tex_sphere_norm.clone()))
                .translated((7.0, 2.0, -1.0))
                .into(),

            SceneNode::from(Geometry::new(Cube, mat_tex_cube_norm.clone()))
                .scaled(2.0)
                .translated((2.0, 0.0, 3.0))
                .into(),
            SceneNode::from(Geometry::new(Sphere, mat_tex_sphere_norm.clone()))
                .translated((2.0, 2.0, 3.0))
                .into(),
        ]).into(),

        lights: vec![
            Light {
                position: Vec3 {x: 0.0, y: 8.0, z: 10.0},
                color: Rgb {r: 0.9, g: 0.9, b: 0.9},
                ..Light::default()
            },
        ],

        ambient: Rgb {r: 0.2, g: 0.2, b: 0.2},
    };

    let cam = CameraSettings {
        eye: (0.0, 8.07551, 23.078941).into(),
        center: (0.0, -2.854475, -16.437334).into(),
        up: Vec3::up(),
        fovy: Radians::from_degrees(22.0),
    };

    let mut image = RgbImage::new(910, 512);

    image.render::<RenderProgress, _>(&scene, cam,
        |uv: Uv| Rgb {r: 0.2, g: 0.4, b: 0.6} * (1.0 - uv.v) + Rgb::blue() * uv.v);

    Ok(image.save("normal-mapping.png")?)
}
