//! Paths through 2D and 3D space
//!
//! See [`SurfacePath`] and [`GlobalPath`].

use fj_math::{Circle, Line, Point, Scalar, Transform, Vector};

/// A path through surface (2D) space
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SurfacePath {
    /// A circle
    Circle(Circle<2>),

    /// A line
    Line(Line<2>),
}

impl SurfacePath {
    /// Build a circle from the given radius
    pub fn circle_from_center_and_radius(
        center: impl Into<Point<2>>,
        radius: impl Into<Scalar>,
    ) -> Self {
        Self::Circle(Circle::from_center_and_radius(center, radius))
    }

    /// Build a line that represents the u-axis of the surface its on
    pub fn u_axis() -> Self {
        let a = Point::origin();
        let b = a + Vector::unit_u();

        let (self_, _) = Self::line_from_points([a, b]);
        self_
    }

    /// Build a line that represents the v-axis of the surface its on
    pub fn v_axis() -> Self {
        let a = Point::origin();
        let b = a + Vector::unit_v();

        let (self_, _) = Self::line_from_points([a, b]);
        self_
    }

    /// Construct a line from two points
    ///
    /// Also returns the coordinates of the points on the path.
    pub fn line_from_points(
        points: [impl Into<Point<2>>; 2],
    ) -> (Self, [Point<1>; 2]) {
        let (line, coords) = Line::from_points(points);
        (Self::Line(line), coords)
    }

    /// Create a line from two points that include line coordinates
    pub fn line_from_points_with_coords(
        points: [(impl Into<Point<1>>, impl Into<Point<2>>); 2],
    ) -> Self {
        Self::Line(Line::from_points_with_line_coords(points))
    }

    /// Convert a point on the path into surface coordinates
    pub fn point_from_path_coords(
        &self,
        point: impl Into<Point<1>>,
    ) -> Point<2> {
        match self {
            Self::Circle(circle) => circle.point_from_circle_coords(point),
            Self::Line(line) => line.point_from_line_coords(point),
        }
    }
}

/// A path through global (3D) space
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum GlobalPath {
    /// A circle
    Circle(Circle<3>),

    /// A line
    Line(Line<3>),
}

impl GlobalPath {
    /// Construct a `GlobalPath` that represents the x-axis
    pub fn x_axis() -> Self {
        Self::Line(Line::from_origin_and_direction(
            Point::origin(),
            Vector::unit_x(),
        ))
    }

    /// Construct a `GlobalPath` that represents the y-axis
    pub fn y_axis() -> Self {
        Self::Line(Line::from_origin_and_direction(
            Point::origin(),
            Vector::unit_y(),
        ))
    }

    /// Construct a `GlobalPath` that represents the z-axis
    pub fn z_axis() -> Self {
        Self::Line(Line::from_origin_and_direction(
            Point::origin(),
            Vector::unit_z(),
        ))
    }

    /// Build a circle from the given radius
    pub fn circle_from_radius(radius: impl Into<Scalar>) -> Self {
        let radius = radius.into();

        Self::Circle(Circle::from_center_and_radius(Point::origin(), radius))
    }

    /// Construct a line from two points
    ///
    /// Also returns the coordinates of the points on the path.
    pub fn line_from_points(
        points: [impl Into<Point<3>>; 2],
    ) -> (Self, [Point<1>; 2]) {
        let (line, coords) = Line::from_points(points);
        (Self::Line(line), coords)
    }

    /// Access the origin of the path's coordinate system
    pub fn origin(&self) -> Point<3> {
        match self {
            Self::Circle(circle) => circle.center() + circle.a(),
            Self::Line(line) => line.origin(),
        }
    }

    /// Convert a point on the path into global coordinates
    pub fn point_from_path_coords(
        &self,
        point: impl Into<Point<1>>,
    ) -> Point<3> {
        match self {
            Self::Circle(circle) => circle.point_from_circle_coords(point),
            Self::Line(line) => line.point_from_line_coords(point),
        }
    }

    /// Convert a vector on the path into global coordinates
    pub fn vector_from_path_coords(
        &self,
        vector: impl Into<Vector<1>>,
    ) -> Vector<3> {
        match self {
            Self::Circle(circle) => circle.vector_from_circle_coords(vector),
            Self::Line(line) => line.vector_from_line_coords(vector),
        }
    }

    /// Transform the path
    #[must_use]
    pub fn transform(self, transform: &Transform) -> Self {
        match self {
            Self::Circle(curve) => {
                Self::Circle(transform.transform_circle(&curve))
            }
            Self::Line(curve) => Self::Line(transform.transform_line(&curve)),
        }
    }
}
