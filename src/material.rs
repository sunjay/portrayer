use std::ops::Range;
use std::sync::Arc;

use rand::{Rng, thread_rng};

use crate::math::{EPSILON, INFINITY, Vec3, Mat3, Uv, Rgb};
use crate::scene::Scene;
use crate::ray::{Ray, RayCast};
use crate::texture::{Texture, NormalMap, TextureSource};

/// Controls the maximum ray recursion depth
const MAX_RECURSION_DEPTH: u32 = 10;

/// Index of refraction of air
pub const AIR_REFRACTION_INDEX: f64 = 1.00;
/// Index of refraction of water
pub const WATER_REFRACTION_INDEX: f64 = 1.33;
/// Index of refraction of window glass
pub const WINDOW_GLASS_REFRACTION_INDEX: f64 = 1.51;
/// Index of refraction of optical glass
pub const OPTICAL_GLASS_REFRACTION_INDEX: f64 = 1.92;
/// Index of refraction of diamond
pub const DIAMOND_REFRACTION_INDEX: f64 = 2.42;

/// Returns the direction of the transmitted / refracted ray (normalized) or None if there is
/// total internal reflection
fn refracted_direction(ray_dir: Vec3, normal: Vec3, refraction_index: f64) -> Option<Vec3> {
    // This formula is from section 13.1 in Fundamentals of Computer Graphics, 4th Ed.

    // The greek letter "eta" is used for the refraction index
    let eta = refraction_index;

    // We are assuming that outside the surface is air
    let eta_outside = AIR_REFRACTION_INDEX;

    let ray_dot_norm = ray_dir.dot(normal);
    let under_sqrt = 1.0 - eta_outside*eta_outside * (1.0 - ray_dot_norm*ray_dot_norm)/(eta*eta);
    if under_sqrt < 0.0 {
        // Total internal reflection
        return None;
    }

    // The direction of refracted / transmitted ray
    // Two variables for the two halfs of the equation
    let refracted_dir_1 = eta_outside * (ray_dir - normal*ray_dot_norm) / eta;
    let refracted_dir_2 = normal * under_sqrt.sqrt();
    Some(refracted_dir_1 - refracted_dir_2)
}

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
    ///
    /// If this material also has a non-zero refraction index, this will be used to blend in the
    /// total color from both the reflected ray and the refracted ray.
    pub reflectivity: f64,
    /// The side length of the glossy reflection rectangle
    pub glossy_side_length: f64,
    /// The index of refraction inside the surface with this material
    ///
    /// It is assumed that the outside of the surface has index of refraction = 1.0 (air)
    pub refraction_index: f64,
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

            // Perturb the reflection ray if we are using glossy reflection
            if self.glossy_side_length > 0.0 {
                // Create a basis u, v from the ideal reflection ray
                // This is a technique for creating a basis from a single vector:
                let offset_vector = if reflect_dir.x.abs() < EPSILON && reflect_dir.y.abs() < EPSILON {
                    // Edge case: reflection direction is aligned with z axis, so the offset in the
                    // else case would result in a collinear vector
                    reflect_dir + Vec3 {x: 0.0, y: 0.1, z: 0.0}
                } else {
                    reflect_dir + Vec3 {x: 0.0, y: 0.0, z: 0.1}
                };
                let u_basis = reflect_dir.cross(offset_vector);
                let v_basis = reflect_dir.cross(u_basis);

                // Generate a random coordinate on the rectangle
                let u_coord = -self.glossy_side_length / 2.0 + rng.gen::<f64>() * self.glossy_side_length;
                let v_coord = -self.glossy_side_length / 2.0 + rng.gen::<f64>() * self.glossy_side_length;

                reflect_dir += u_coord*u_basis + v_coord*v_basis;
            }

            // Add reflection via recursive ray tracing
            let reflected_ray = Ray::new(hit_point, reflect_dir);
            let reflected_color = reflected_ray.color(scene, background, recursion_depth + 1);

            // This code is translated from pseudo code in Section 13.1 of
            // Fundamentals of Computer Graphics, 4th Ed.
            if self.refraction_index > 0.0 {
                // Dielectric material

                // The reflectivity of a dielectric varies with the incident angle according to the
                // Fresnel equations. We use the Schlick approximation which uses the cosine of the
                // incident angle.
                let refract_dir_cos_incident = if ray_dir.dot(normal) < 0.0 {
                    // Ray is going into the surface

                    // Refracted / transmitted ray
                    let refract_dir = refracted_direction(ray_dir, normal, self.refraction_index)
                        .expect("bug: should not have total internal reflection when casting inside surface");
                    // Incident angle here is the angle between the ray and the normal. Ray is
                    // reversed because it is currently pointing towards the surface and we want
                    // the other angle.
                    let cos_incident = (-ray_dir).dot(normal);

                    Some((refract_dir, cos_incident))
                } else if let Some(refract_dir) = refracted_direction(ray_dir, -normal, 1.0/self.refraction_index) {
                    // Ray is heading outside the surface

                    // Since the ray is coming from inside the surface, the light (which is on the
                    // outside) is actually incident with the refracted ray. Note that we don't
                    // need to flip the refracted direction in this branch because it is already
                    // pointing away from the surface (unlike the ray direction in the previous
                    // case).
                    let cos_incident = refract_dir.dot(normal);

                    Some((refract_dir, cos_incident))
                } else {
                    // Total internal reflection

                    // Since there is only reflection, this code is the same as the reflective-only case
                    color += self.reflectivity * reflected_color;

                    // Finish processing if this is the case
                    None
                };

                // Only continue if there was not total internal reflection
                if let Some((refract_dir, cos_incident)) = refract_dir_cos_incident {
                    // Compute the reflectivity using the Schlick approximation

                    // The reflectivity at normal incidence
                    // r0 = (eta - 1)^2/(eta + 1)^2
                    let r0 = (self.refraction_index - 1.0)*(self.refraction_index - 1.0);
                    let r0 = r0 / ((self.refraction_index + 1.0)*(self.refraction_index + 1.0));
                    // The reflectivity according to the approximation, distinct from the property
                    // in the material
                    let reflectivity = r0 + (1.0 - r0) * (1.0 - cos_incident).powi(5);

                    // By conservation of energy, the energy not transmitted/refracted is reflected
                    let transmittance = 1.0 - reflectivity;

                    // Cast the transmitted ray and determine the color
                    let refracted_ray = Ray::new(hit_point, refract_dir);
                    let refracted_color = refracted_ray.color(scene, background, recursion_depth + 1);

                    // The total color uses the result of Fresnel/Schlick to mix the reflected and
                    // refracted/transmitted colors
                    let total_color = reflectivity * reflected_color + transmittance * refracted_color;
                    // Mix in the total color using the material reflectivity coefficient
                    color += self.reflectivity * total_color;
                }

            } else {
                // Reflective-only material

                color += self.reflectivity * reflected_color;
            }
        }

        color
    }
}
