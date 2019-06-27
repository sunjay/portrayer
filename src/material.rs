use std::ops::Range;

use crate::math::{EPSILON, INFINITY, Vec3, Rgb};
use crate::scene::Scene;
use crate::ray::Ray;

// Controls the maximum ray recursion depth
const MAX_RECURSION_DEPTH: u32 = 10;

#[derive(Debug, Clone)]
pub struct Material {
    /// The diffuse reflection constant
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
}

impl Material {
    /// Compute the color of a ray intersection using the lighting model of this material, possibly
    /// casting further rays to simulate things like reflection/refraction/etc.
    pub fn hit_color(
        &self,
        scene: &Scene,
        background: Rgb,
        ray_dir: Vec3,
        hit_point: Vec3,
        normal: Vec3,
        recursion_depth: u32,
    ) -> Rgb {
        if recursion_depth > MAX_RECURSION_DEPTH {
            return background;
        }

        // Vector from hit point to the eye (ray origin)
        // Note that this is the same as -ray.direction() since the ray intersects with the
        // hit point
        let view = -ray_dir;
        // Surface normal of hit point
        // Need to normalize because the normal provided is not guaranteed to be a unit vector
        let normal = normal.normalized();

        // Start with the ambient color since that is always added
        // Need to multiply by the diffuse color because the ambient light is still affected by the
        // color of the object
        let mut color = scene.ambient * self.diffuse;
        for light in &scene.lights {
            // Vector from hit point to the light source
            // NOTE: This is **flipped** from the actual direction of light from the light source
            let hit_to_light = light.position - hit_point;

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
            if shadow_ray.cast(&*scene.root, &mut shadow_t_range).is_none() {
                // Want the max diffuse when the light is directly aligned with the surface normal.
                // Using normal.dot(light_dir) == cos(angle between normal and light)
                // we can accomplish this effect.
                // Need to max with zero so we can ignore backface contributions
                let normal_light = normal.dot(light_dir).max(0.0);
                let diffuse = self.diffuse * light.color * normal_light;

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
            let reflect_dir = ray_dir - normal * 2.0 * ray_dir.dot(normal);

            // Add reflection via recursive ray tracing
            let reflected_ray = Ray::new(hit_point, reflect_dir);
            let reflected_color = reflected_ray.color(scene, background, recursion_depth + 1);
            color += reflected_color * self.reflectivity;
        }

        color
    }
}
