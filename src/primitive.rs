mod sphere;
mod triangle;
mod mesh;
mod infinite_plane;
mod cube;
mod plane;
mod cylinder;
mod cone;
mod torus;

pub use sphere::*;
pub use triangle::*;
pub use mesh::*;
pub use cube::*;
pub use plane::*;
pub use cylinder::*;
pub use cone::*;
pub use torus::*;

// Internal-use only
pub(crate) use infinite_plane::*;

use std::ops::Range;

use crate::bounding_box::{BoundingBox, Bounds};
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

        impl Bounds for $name {
            fn bounds(&self) -> BoundingBox {
                use $name::*;
                match self {
                    $($variant(prim) => prim.bounds()),*
                }
            }
        }

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
    #[derive(Debug, Clone, PartialEq)]
    pub enum Primitive {
        Sphere(Sphere),
        Triangle(Triangle),
        Mesh(Mesh),
        // InfinitePlane cannot be part of this enum because it is infinite and that means that
        // there is no logical implementation of the Bounds trait for InfinitePlane
        Plane(Plane),
        Cube(Cube),
        Cylinder(Cylinder),
        Cone(Cone),
    }
}
