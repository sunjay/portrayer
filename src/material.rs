use std::ops::Range;
use std::sync::Arc;

use rand::{Rng, thread_rng};

use crate::math::{EPSILON, INFINITY, Vec3, Vec3Ext, Mat3, Uv, Rgb};
use crate::scene::Scene;
use crate::ray::{Ray, RayCast};
use crate::texture::{Texture, NormalMap, TextureSource};

// Controls the maximum ray recursion depth
const MAX_RECURSION_DEPTH: u32 = 10;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Material {
    /// The diffuse color and intensity of the material
    ///
    /// Ignored if a texture is provided
    pub diffuse: Rgb,
    /// The specular reflection constant
    pub specular: Rgb,
    /// The Phong exponent (shininess)
    ///
    /// * 10 - "eggshell"
    /// * 100 - mildly shiny
    /// * 1000 - really glossy
    /// * 10,000 - nearly mirror-like
    pub shininess: f64,
    /// The reflectivity of this material, used to blend in the color from the reflected ray
    pub reflectivity: f64,
    /// The side length of the glossy reflection rectangle
    pub glossy_side_length: f64,
    /// The texture to sample the diffuse color from
    pub texture: Option<Arc<Texture>>,
    /// The texture to sample the shading normal from
    pub normals: Option<Arc<NormalMap>>,
}

impl Material {
    /// Compute the color of a ray intersection using the lighting model of this material, possibly
    /// casting further rays to simulate things like reflection/refraction/etc.
    pub fn hit_color<R: RayCast>(
        &self,
        scene: &Scene<R>,
        background: Rgb,
        ray_dir: Vec3,
        hit_point: Vec3,
        normal: Vec3,
        tex_coord: Option<Uv>,
        normal_map_transform: Option<Mat3>,
        recursion_depth: u32,
    ) -> Rgb {
        if recursion_depth > MAX_RECURSION_DEPTH {
            return background;
        }

        let mut rng = thread_rng();

        // Vector from hit point to the eye (ray origin)
        // Note that this is the same as -ray.direction() since the ray intersects with the
        // hit point
        let view = -ray_dir;

        // Surface normal of hit point
        //
        // The code below relies on this being normalized
        let normal = match &self.normals {
            // Need to normalize because the normal provided is not guaranteed to be a unit vector
            None => normal.normalized(),
            // Need both the texture coordinate and the normal map transform to be present
            Some(tex) => match (tex_coord, normal_map_transform) {
                (Some(tex_coord), Some(norm_trans)) => {
                    let tex_norm = tex.normal_at(tex_coord);
                    // Need to normalize because normal from texture map may not be normalized and
                    // norm_trans may also potentially result in a non-normalized vector
                    norm_trans * tex_norm.normalized()
                },
                _ => panic!("Normal/Texture mapping is not supported for this primitive!"),
            },
        };

        let diffuse_color = match &self.texture {
            None => self.diffuse,
            Some(tex) => match tex_coord {
                Some(tex_coord) => tex.at(tex_coord),
                None => panic!("Texture mapping is not supported for this primitive!"),
            },
        };

        // Start with the ambient color since that is always added
        // Need to multiply by the diffuse color because the ambient light is still affected by the
        // color of the object
        let mut color = scene.ambient * diffuse_color;
        for light in &scene.lights {
            let light_pos = if light.area.is_empty() {
                light.position
            } else {
                // Check if we are behind this light
                if light.area.normal().dot(normal) > 0.0 {
                    // Behind the light, do not count its contribution
                    continue;
                }
                light.sample_position(&mut rng)
            };

            // Vector from hit point to the light source
            // NOTE: This is **flipped** from the actual direction of light from the light source
            let hit_to_light = light_pos - hit_point;

            // The distance r between the light source and the hit point. Used to
            // attenuate the light.
            let light_dist = hit_to_light.magnitude();

            // The direction towards the light from the hit point
            // Reusing the already calculated magnitude to normalize
            let light_dir = hit_to_light / light_dist;

            // attenuation - based on the light falloff values
            let attenuation = light.falloff.at_distance(light_dist);

            // Cast a ray to the light to determine if anything is between this point and the light
            // If there is something, this point must be in "shadow" since it cannot be hit by the
            // light directly.
            let shadow_ray = Ray::new(hit_point, light_dir);
            // The EPSILON helps avoid self-intersections (and "shadow acne")
            let mut shadow_t_range = Range {start: EPSILON, end: INFINITY};

            // Only add diffuse if not shadowed by another object
            if scene.root.ray_cast(&shadow_ray, &mut shadow_t_range).is_none() {
                // Want the max diffuse when the light is directly aligned with the surface normal.
                // Using normal.dot(light_dir) == cos(angle between normal and light)
                // we can accomplish this effect.
                // Need to max with zero so we can ignore backface contributions
                let normal_light = normal.dot(light_dir).max(0.0);
                let diffuse = diffuse_color * light.color * normal_light;

                // Check if there is any specular component of the material. Allows us to avoid
                // some calculations for non-specular materials.
                let specular = if self.specular.iter().any(|&v| v > EPSILON) {
                    // half-vector -- halway between the light vector and the view vector. If this
                    // is aligned with the normal, we have angle of incidence == angle of
                    // reflection (mirror reflection)
                    // Since normal.dot(half) == cos(angle between normal and half vector),
                    // this will give us 1.0 when we have perfect mirror reflection
                    // That produces the highest specular value when our light is perfectly aligned
                    let half = (view + light_dir).normalized();

                    // Need to multiply shininess by 4 because the angle in Blinn-Phong is much
                    // smaller than in Phong so it needs that extra boost in order to work the same
                    // with the same values
                    // Source: https://learnopengl.com/Advanced-Lighting/Advanced-Lighting
                    let normal_half_shiny = normal.dot(half).max(0.0).powf(4.0 * self.shininess);

                    self.specular * light.color * normal_half_shiny
                } else {
                    Rgb::from(0.0)
                };

                // Attenuate light contribution before adding to the final color
                color += (diffuse + specular) / attenuation;
            }
        }

        // Check if there is any reflective component of the material.
        // Allows us to avoid some recursion for non-reflective materials.
        if self.reflectivity > 0.0 {
            // r = v - 2N(v dot N) where v = ray direction, N = normal
            let mut reflect_dir = ray_dir - normal * 2.0 * ray_dir.dot(normal);
            if self.glossy_side_length > 0.0 {
                // Create a basis u, v from the ideal reflection ray
                let (_, u_basis, v_basis) = reflect_dir.orthonormal_basis();

                // Generate a random coordinate on the rectangle
                let u_coord = -self.glossy_side_length / 2.0 + rng.gen::<f64>() * self.glossy_side_length;
                let v_coord = -self.glossy_side_length / 2.0 + rng.gen::<f64>() * self.glossy_side_length;

                reflect_dir += u_coord*u_basis + v_coord*v_basis;
            }

            // Add reflection via recursive ray tracing
            let reflected_ray = Ray::new(hit_point, reflect_dir);
            let reflected_color = reflected_ray.color(scene, background, recursion_depth + 1);
            color += reflected_color * self.reflectivity;
        }

        color
    }
}
