//! # Path approximation
//!
//! Since paths are infinite (even circles have an infinite coordinate space,
//! even though they connect to themselves in global coordinates), a range must
//! be provided to approximate them. The approximation then returns points
//! within that range.
//!
//! The boundaries of the range are not included in the approximation. This is
//! done, to give the caller (who knows the boundary anyway) more options on how
//! to further process the approximation.
//!
//! ## Determinism
//!
//! Path approximation is carefully designed to produce a deterministic result
//! for the combination of a given path and a given tolerance, regardless of
//! what the range is. This is done to prevent invalid meshes from being
//! generated.
//!
//! In specific terms, this means there is an infinite set of points that
//! approximates a path, and that set is deterministic for a given combination
//! of path and tolerance. The range that defines where the path is approximated
//! only influences the result in two ways:
//!
//! 1. It controls which points from the infinite set are actually computed.
//! 2. It defines the order in which the computed points are returned.
//!
//! As a result, path approximation is guaranteed to generate points that can
//! fit together in a valid mesh, no matter which ranges of a path are being
//! approximated, and how many times.

use std::iter;

use fj_math::{Circle, Point, Scalar, Sign};

use crate::geometry::{BoundaryOnCurve, GlobalPath, SurfacePath};

use super::{Approx, Tolerance};

impl Approx for (&SurfacePath, BoundaryOnCurve) {
    type Approximation = Vec<(Point<1>, Point<2>)>;
    type Cache = ();

    fn approx_with_cache(
        self,
        tolerance: impl Into<Tolerance>,
        (): &mut Self::Cache,
    ) -> Self::Approximation {
        let (path, range) = self;

        match path {
            SurfacePath::Circle(circle) => {
                approx_circle(circle, range, tolerance.into())
            }
            SurfacePath::Line(_) => vec![],
        }
    }
}

impl Approx for (GlobalPath, BoundaryOnCurve) {
    type Approximation = Vec<(Point<1>, Point<3>)>;
    type Cache = ();

    fn approx_with_cache(
        self,
        tolerance: impl Into<Tolerance>,
        (): &mut Self::Cache,
    ) -> Self::Approximation {
        let (path, range) = self;

        match path {
            GlobalPath::Circle(circle) => {
                approx_circle(&circle, range, tolerance.into())
            }
            GlobalPath::Line(_) => vec![],
        }
    }
}

/// Approximate a circle
///
/// `tolerance` specifies how much the approximation is allowed to deviate
/// from the circle.
fn approx_circle<const D: usize>(
    circle: &Circle<D>,
    boundary: impl Into<BoundaryOnCurve>,
    tolerance: Tolerance,
) -> Vec<(Point<1>, Point<D>)> {
    let boundary = boundary.into();

    let params = PathApproxParams::for_circle(circle, tolerance);
    let mut points = Vec::new();

    for point_curve in params.points(boundary) {
        let point_global = circle.point_from_circle_coords(point_curve);
        points.push((point_curve, point_global));
    }

    points
}

struct PathApproxParams {
    increment: Scalar,
}

impl PathApproxParams {
    pub fn for_circle<const D: usize>(
        circle: &Circle<D>,
        tolerance: impl Into<Tolerance>,
    ) -> Self {
        let radius = circle.a().magnitude();

        let num_vertices_to_approx_full_circle = Scalar::max(
            Scalar::PI
                / (Scalar::ONE - (tolerance.into().inner() / radius)).acos(),
            3.,
        )
        .ceil();

        let increment = Scalar::TAU / num_vertices_to_approx_full_circle;

        Self { increment }
    }

    pub fn increment(&self) -> Scalar {
        self.increment
    }

    pub fn points(
        &self,
        boundary: impl Into<BoundaryOnCurve>,
    ) -> impl Iterator<Item = Point<1>> + '_ {
        let boundary = boundary.into();

        let [a, b] = boundary.inner.map(|point| point.t / self.increment());
        let direction = (b - a).sign();
        let [min, max] = if a < b { [a, b] } else { [b, a] };

        // We can't generate a point exactly at the boundaries of the range as
        // part of the approximation. Make sure we stay inside the range.
        let min = min.floor() + 1.;
        let max = max.ceil() - 1.;

        let [start, end] = match direction {
            Sign::Negative => [max, min],
            Sign::Positive | Sign::Zero => [min, max],
        };

        let mut i = start;
        iter::from_fn(move || {
            let is_finished = match direction {
                Sign::Negative => i < end,
                Sign::Positive | Sign::Zero => i > end,
            };

            if is_finished {
                return None;
            }

            let t = self.increment() * i;
            i += direction.to_scalar();

            Some(Point::from([t]))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use fj_math::{Circle, Point, Scalar};

    use crate::algorithms::approx::{path::BoundaryOnCurve, Tolerance};

    use super::PathApproxParams;

    #[test]
    fn increment_for_circle() {
        test_increment(1., 0.5, 3.);
        test_increment(1., 0.1, 7.);
        test_increment(1., 0.01, 23.);

        fn test_increment(
            radius: impl Into<Scalar>,
            tolerance: impl Into<Tolerance>,
            expected_num_vertices: impl Into<Scalar>,
        ) {
            let circle = Circle::from_center_and_radius([0., 0.], radius);
            let params = PathApproxParams::for_circle(&circle, tolerance);

            let expected_increment = Scalar::TAU / expected_num_vertices;
            assert_eq!(params.increment(), expected_increment);
        }
    }

    #[test]
    fn points_for_circle() {
        // At the chosen values for radius and tolerance (see below), the
        // increment is `PI / 4`, so ~1.57.

        // Empty range
        let empty: [Scalar; 0] = [];
        test_path([[0.], [0.]], empty);

        // Ranges contain all generated points. Start is before the first
        // increment and after the last one in each case.
        test_path([[0.], [TAU]], [1., 2., 3.]);
        test_path([[1.], [TAU]], [1., 2., 3.]);
        test_path([[0.], [TAU - 1.]], [1., 2., 3.]);

        // Here the range is restricted to cut of the first or last increment.
        test_path([[2.], [TAU]], [2., 3.]);
        test_path([[0.], [TAU - 2.]], [1., 2.]);

        // And everything again, but in reverse.
        test_path([[TAU], [0.]], [3., 2., 1.]);
        test_path([[TAU], [1.]], [3., 2., 1.]);
        test_path([[TAU - 1.], [0.]], [3., 2., 1.]);
        test_path([[TAU], [2.]], [3., 2.]);
        test_path([[TAU - 2.], [0.]], [2., 1.]);

        fn test_path(
            boundary: impl Into<BoundaryOnCurve>,
            expected_coords: impl IntoIterator<Item = impl Into<Scalar>>,
        ) {
            // Choose radius and tolerance such, that we need 4 vertices to
            // approximate a full circle. This is the lowest number that we can
            // still cover all the edge cases with
            let radius = 1.;
            let tolerance = 0.375;

            let circle = Circle::from_center_and_radius([0., 0.], radius);
            let params = PathApproxParams::for_circle(&circle, tolerance);

            let points = params.points(boundary).collect::<Vec<_>>();

            let expected_points = expected_coords
                .into_iter()
                .map(|i| Point::from([params.increment() * i]))
                .collect::<Vec<_>>();
            assert_eq!(points, expected_points);
        }
    }
}
