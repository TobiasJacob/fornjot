use crate::{geometry::vertices::Vertices as _, math::Point};

/// Compute the bounding volume of a shape
pub trait BoundingVolume {
    /// Compute the [`Aabb`] of a shape
    fn aabb(&self) -> Aabb;
}

impl BoundingVolume for fj::Shape {
    fn aabb(&self) -> Aabb {
        match self {
            Self::Shape2d(shape) => shape.aabb(),
            Self::Shape3d(shape) => shape.aabb(),
        }
    }
}

impl BoundingVolume for fj::Shape2d {
    fn aabb(&self) -> Aabb {
        match self {
            Self::Square(shape) => shape.aabb(),
        }
    }
}

impl BoundingVolume for fj::Shape3d {
    fn aabb(&self) -> Aabb {
        match self {
            Self::Cube(shape) => shape.aabb(),
            Self::Sweep(_) => {
                // TASK: Implement.
                todo!()
            }
        }
    }
}

impl BoundingVolume for fj::Square {
    fn aabb(&self) -> Aabb {
        let mut vertices = self.vertices();

        // Can't panic. We know a square has at least one vertex.
        let vertex = vertices.pop().unwrap();

        // Seed values with one of the square's vertices.
        let mut min_x = vertex.x;
        let mut max_x = vertex.x;
        let mut min_y = vertex.y;
        let mut max_y = vertex.y;

        for vertex in vertices {
            if vertex.x < min_x {
                min_x = vertex.x;
            }
            if vertex.x > max_x {
                max_x = vertex.x;
            }
            if vertex.y < min_y {
                min_y = vertex.y;
            }
            if vertex.y > max_y {
                max_y = vertex.y;
            }
        }

        Aabb {
            min: [min_x, min_y, 0.0].into(),
            max: [max_x, max_y, 0.0].into(),
        }
    }
}

impl BoundingVolume for fj::Cube {
    fn aabb(&self) -> Aabb {
        let mut vertices = self.vertices();

        // Can't panic. We know a cube has at least one vertex.
        let vertex = vertices.pop().unwrap();

        // Seed values with one of the cube's vertices.
        let mut min_x = vertex.x;
        let mut max_x = vertex.x;
        let mut min_y = vertex.y;
        let mut max_y = vertex.y;
        let mut min_z = vertex.z;
        let mut max_z = vertex.z;

        for vertex in vertices {
            if vertex.x < min_x {
                min_x = vertex.x;
            }
            if vertex.x > max_x {
                max_x = vertex.x;
            }
            if vertex.y < min_y {
                min_y = vertex.y;
            }
            if vertex.y > max_y {
                max_y = vertex.y;
            }
            if vertex.z < min_z {
                min_z = vertex.z;
            }
            if vertex.z > max_z {
                max_z = vertex.z;
            }
        }

        Aabb {
            min: [min_x, min_y, min_z].into(),
            max: [max_x, max_y, max_z].into(),
        }
    }
}

/// An axis-aligned bounding box
#[derive(Debug)]
pub struct Aabb {
    /// Minimum point of the axis-aligned bounding box
    pub min: Point,

    /// Maximum point of the axis-aligned bounding box
    pub max: Point,
}
