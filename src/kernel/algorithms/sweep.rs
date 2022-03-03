use crate::{
    kernel::topology::{
        faces::{Face, Faces},
        Shape,
    },
    math::{Scalar, Transform, Vector},
};

use super::{approximation::Approximation, transform::transform_face};

/// Create a new shape by sweeping an existing one
pub fn sweep_shape(
    original: &Shape,
    path: Vector<3>,
    tolerance: Scalar,
) -> Shape {
    let mut shape = Shape::new();

    let translation = Transform::translation(path);

    let mut bottom_faces = Vec::new();
    let mut top_faces = Vec::new();
    let mut side_faces = Vec::new();

    for face in &original.faces.0 {
        bottom_faces.push(face.clone());
        top_faces.push(transform_face(face, &translation, &mut shape));
    }

    for cycle in &original.edges.cycles {
        let approx = Approximation::for_cycle(cycle, tolerance);

        // This will only work correctly, if the cycle consists of one edge. If
        // there are more, this will create some kind of weird face chimera, a
        // single face to represent all the side faces.

        let mut quads = Vec::new();
        for segment in approx.segments {
            let [v0, v1] = segment.points();
            let [v3, v2] = {
                let segment =
                    Transform::translation(path).transform_segment(&segment);
                segment.points()
            };

            quads.push([v0, v1, v2, v3]);
        }

        let mut side_face = Vec::new();
        for [v0, v1, v2, v3] in quads {
            side_face.push([v0, v1, v2].into());
            side_face.push([v0, v2, v3].into());
        }

        side_faces.push(Face::Triangles(side_face));
    }

    let mut faces = Vec::new();
    faces.extend(bottom_faces);
    faces.extend(top_faces);
    faces.extend(side_faces);

    shape.faces = Faces(faces);

    shape
}

#[cfg(test)]
mod tests {
    use crate::{
        kernel::{
            geometry::{surfaces::Swept, Surface},
            topology::{
                edges::{Edge, Edges},
                faces::Face,
                Shape,
            },
        },
        math::{Point, Scalar, Vector},
    };

    use super::sweep_shape;

    #[test]
    fn sweep() {
        let sketch = Triangle::new([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);

        let swept = sweep_shape(
            &sketch.shape,
            Vector::from([0., 0., 1.]),
            Scalar::from_f64(0.),
        );

        assert!(swept.faces.0.contains(&sketch.face));
    }

    pub struct Triangle {
        shape: Shape,
        face: Face,
    }

    impl Triangle {
        fn new([a, b, c]: [impl Into<Point<3>>; 3]) -> Self {
            let mut shape = Shape::new();

            let a = shape.vertices().create(a.into());
            let b = shape.vertices().create(b.into());
            let c = shape.vertices().create(c.into());

            let ab = Edge::line_segment([a, b]);
            let bc = Edge::line_segment([b, c]);
            let ca = Edge::line_segment([c, a]);

            let abc = Face::Face {
                surface: Surface::Swept(Swept::plane_from_points(
                    [a, b, c].map(|vertex| vertex.point().canonical()),
                )),
                edges: Edges::single_cycle([ab, bc, ca]),
            };

            shape.faces.0.push(abc.clone());

            Self { shape, face: abc }
        }
    }
}
