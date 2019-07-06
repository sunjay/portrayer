//! The vek crate that we use to provide math primitives is very generic, but we will always want
//! to use it with floats. This module exports type aliases that allow us to not have to specify
//! that we are using "f64" all the time.

pub use std::f64::INFINITY;

/// This constant is a "fudge factor" used to account for floating point error in calculations.
/// It is different from machine epsilon because we accumulate quite a bit more error than that.
pub const EPSILON: f64 = 0.00001;

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

/// A quadratic equation solver: a*x^2 + b*x + c
#[derive(Debug, Clone, Copy)]
pub struct Quadratic {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Quadratic {
    /// Lazily generates up to 2 solutions in order from smallest to largest
    pub fn solve(self) -> impl Iterator<Item=f64> {
        let Quadratic {a, b, c} = self;
        let discriminant = b*b - 4.0*a*c;

        // Lazily generate the solutions by creating closures that compute them as needed
        // Using an alternate form of the quadratic formula that is prone to fewer numerical errors
        let sol1 = move || 2.0*c / (-b + discriminant.sqrt());
        let sol2 = move || 2.0*c / (-b - discriminant.sqrt());

        //TODO: Replace this mess with std::iter::once_with() when that is stable
        //once_with(move || if discriminant >= -EPSILON { Some(sol1()) } else { None })
        //    .chain(once_with(move || if discriminant > 0.0 { Some(sol2()) } else { None }))
        if discriminant >= -EPSILON { Some(()) } else { None }
            .into_iter()
            .map(move |_| sol1())
            .chain(if discriminant > 0.0 { Some(()) } else { None }
                .into_iter()
                .map(move |_| sol2()))
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
            let solutions = equation.solve().collect::<Vec<_>>();

            let expected: &[f64] = &[$($sol),*];

            assert_eq!(expected.len(), solutions.len(),
                "got {} solution(s) when expected {} solution(s):\n\texpected: {:?}\n\tactual: {:?}",
                solutions.len(), expected.len(), expected, solutions);
            for (expected, actual) in expected.into_iter().zip(solutions) {
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
}
