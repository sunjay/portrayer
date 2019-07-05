mod sphere;
mod triangle;
mod mesh;
mod plane;
mod cube;
mod finite_plane;

pub use sphere::*;
pub use triangle::*;
pub use mesh::*;
pub use plane::*;
pub use cube::*;
pub use finite_plane::*;

use std::ops::Range;

use crate::ray::{Ray, RayHit, RayIntersection};

// This macro generates boilerplate code for the primitives and makes it easier to
// add as many as needed without having to write the same thing over and over again.
macro_rules! primitive_enum {
    ($(#[$m:meta])* pub enum $name:ident {
        $($variant:ident ( $primtype:ident ),)*
    }) => {
        $(#[$m])*
        pub enum $name {
            $($variant($primtype),)*
        }

        $(
            impl From<$primtype> for $name {
                fn from(prim: $primtype) -> Self {
                    $name::$variant(prim)
                }
            }
        )*

        impl RayHit for $name {
            fn ray_hit(&self, ray: &Ray, t_range: &Range<f64>) -> Option<RayIntersection> {
                use $name::*;
                match self {
                    $($variant(prim) => prim.ray_hit(ray, t_range)),*
                }
            }
        }
    };
}

// All of the impls will be generated just based on this declaration
primitive_enum! {
    #[derive(Debug)]
    pub enum Primitive {
        Sphere(Sphere),
        Triangle(Triangle),
        Mesh(Mesh),
        Plane(Plane),
        FinitePlane(FinitePlane),
        Cube(Cube),
    }
}
