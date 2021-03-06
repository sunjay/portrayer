//! Math-related utilities/constants/type definitions.
//!
//! The vek crate that we use to provide math primitives is very generic, but we will always want
//! to use it with floats. This module exports type aliases that allow us to not have to specify
//! that we are using "f64" all the time.

pub use std::f64::INFINITY;

use std::ops::Range;

use roots::Roots;

/// This constant is a "fudge factor" used to account for floating point error in calculations.
/// It is different from machine epsilon because we accumulate quite a bit more error than that.
pub const EPSILON: f64 = 0.00001;

// Gamma correction is used to ensure that image colors are closer to what we want them to be.
// This gamma value is the same as Blender and is also in the source below.
// Source: https://learnopengl.com/Advanced-Lighting/Gamma-Correction
pub const GAMMA: f64 = 2.2;

pub type Vec2 = vek::Vec2<f64>;
pub type Vec3 = vek::Vec3<f64>;
pub type Vec4 = vek::Vec4<f64>;

pub type Mat2 = vek::Mat2<f64>;
pub type Mat3 = vek::Mat3<f64>;
pub type Mat4 = vek::Mat4<f64>;

pub type Rgba = vek::Rgba<f64>;
pub type Rgb = vek::Rgb<f64>;

pub type Uv = vek::Uv<f64>;

/// This is an "extension trait". It allows me to add methods to structs I did not define.
pub trait Vec3Ext {
    /// Interprets this as a point and applies the given transformation matrix
    fn transformed_point(self, trans: Mat4) -> Self;

    /// Interprets this as a direction and applies the given transformation matrix
    fn transformed_direction(self, trans: Mat4) -> Self;
}

impl Vec3Ext for Vec3 {
    fn transformed_point(self, trans: Mat4) -> Self {
        Vec3::from(trans * Vec4::from_point(self))
    }

    fn transformed_direction(self, trans: Mat4) -> Self {
        Vec3::from(trans * Vec4::from_direction(self))
    }
}

/// A "newtype" to represent a value with the unit "radians"
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Radians(f64);

impl Radians {
    pub fn from_degrees(value: f64) -> Self {
        Radians(value.to_radians())
    }

    pub fn from_radians(value: f64) -> Self {
        Radians(value)
    }

    /// Returns the actual value stored in this struct (in radians)
    pub fn get(self) -> f64 {
        self.0
    }
}

#[derive(Debug)]
pub struct Solutions(Roots<f64>);

impl PartialEq<[f64]> for Solutions {
    fn eq(&self, other: &[f64]) -> bool {
        self.0.as_ref() == other
    }
}

impl Solutions {
    /// Returns the number of solutions
    pub fn len(&self) -> usize {
        self.0.as_ref().len()
    }

    /// Provides an iterator over the solutions, from smallest to largest
    pub fn iter(&self) -> impl Iterator<Item=f64> + '_ {
        self.0.as_ref().iter().cloned()
    }

    /// Finds the smallest solution in the given range
    pub fn find_in_range(&self, range: &Range<f64>) -> Option<f64> {
        self.0.as_ref().iter().find(|t| range.contains(t)).cloned()
    }
}

/// A quadratic equation solver for: a*x^2 + b*x + c = 0
#[derive(Debug, Clone, Copy)]
pub struct Quadratic {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Quadratic {
    /// Solve the given equation and return up to two solutions
    pub fn solve(self) -> Solutions {
        let Quadratic {a, b, c} = self;

        Solutions(roots::find_roots_quadratic(a, b, c))
    }
}

/// A quartic equation solver for: a*x^4 + b*x^3 + c*x^2 + d*x + e = 0
#[derive(Debug, Clone, Copy)]
pub struct Quartic {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
}

impl Quartic {
    /// Solve the given equation and return up to two solutions
    pub fn solve(self) -> Solutions {
        let Quartic {a, b, c, d, e} = self;

        Solutions(roots::find_roots_quartic(a, b, c, d, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_approx_eq::assert_approx_eq;

    // This macro allows us to write quadratic equations in mathematical notation and test that
    // the solutions are all correct
    macro_rules! test_quadratic {
        ($a:literal * x ^ 2 + $b:literal * x + $c:literal = 0, [ $($sol:expr),* ]) => {
            let equation = Quadratic {a: $a, b: $b, c: $c};
            let solutions = equation.solve();

            let expected: &[f64] = &[$($sol),*];

            assert_eq!(expected.len(), solutions.len(),
                "got {} solution(s) when expected {} solution(s):\n\texpected: {:?}\n\tactual: {:?}",
                solutions.len(), expected.len(), expected, solutions);
            for (expected, actual) in expected.iter().zip(solutions.iter()) {
                assert_approx_eq!(expected, actual);
            }
        };
    }

    #[test]
    fn solve_quadratic_equations() {
        // discriminant > 0
        test_quadratic!(2.0*x^2 + 8.0*x + 3.0 = 0,
            // Solutions ordered from smallest to largest
            [-2.0 - (5.0/2.0f64).sqrt(), (5.0/2.0f64).sqrt() - 2.0]);
        // discriminant == 0
        test_quadratic!(4.0*x^2 + -4.0*x + 1.0 = 0,
            [0.5]);
        // discriminant < 0
        test_quadratic!(3.0*x^2 + 4.0*x + 2.0 = 0,
            []);
    }

    #[test]
    fn solution_order() {
        // Since the denominator is negative, figuring out the smallest t value is more complex
        test_quadratic!(-2.0*x^2 + 8.0*x + 3.0 = 0,
            // Solutions ordered from smallest to largest
            [2.0 - (11.0/2.0f64).sqrt(), 2.0 + (11.0/2.0f64).sqrt()]);
    }
}
