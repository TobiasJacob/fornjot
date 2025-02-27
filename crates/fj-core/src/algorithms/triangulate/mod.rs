//! Shape triangulation

mod delaunay;
mod polygon;

use fj_interop::mesh::Mesh;
use fj_math::Point;

use self::polygon::Polygon;

use super::approx::{face::FaceApprox, Approx, Tolerance};

/// Triangulate a shape
pub trait Triangulate: Sized {
    /// Triangulate the shape
    fn triangulate(self) -> Mesh<Point<3>> {
        let mut mesh = Mesh::new();
        self.triangulate_into_mesh(&mut mesh);
        mesh
    }

    /// Triangulate a partial shape into the provided mesh
    ///
    /// This is a low-level method, intended for implementation of
    /// `Triangulate`. Most callers should prefer [`Triangulate::triangulate`].
    fn triangulate_into_mesh(self, mesh: &mut Mesh<Point<3>>);
}

impl<T> Triangulate for (T, Tolerance)
where
    T: Approx,
    T::Approximation: IntoIterator<Item = FaceApprox>,
{
    fn triangulate_into_mesh(self, mesh: &mut Mesh<Point<3>>) {
        let (approx, tolerance) = self;

        let approx = approx.approx(tolerance);

        for approx in approx {
            approx.triangulate_into_mesh(mesh);
        }
    }
}

impl Triangulate for FaceApprox {
    fn triangulate_into_mesh(self, mesh: &mut Mesh<Point<3>>) {
        let face_as_polygon = Polygon::new()
            .with_exterior(
                self.exterior
                    .points()
                    .into_iter()
                    .map(|point| point.local_form),
            )
            .with_interiors(self.interiors.iter().map(|interior| {
                interior.points().into_iter().map(|point| point.local_form)
            }));

        let cycles = [self.exterior].into_iter().chain(self.interiors);
        let mut triangles =
            delaunay::triangulate(cycles, self.coord_handedness);
        triangles.retain(|triangle| {
            face_as_polygon
                .contains_triangle(triangle.map(|point| point.point_surface))
        });

        let color = self.color.unwrap_or_default();

        for triangle in triangles {
            let points = triangle.map(|point| point.point_global);
            mesh.push_triangle(points, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use fj_interop::mesh::Mesh;
    use fj_math::{Point, Scalar};

    use crate::{
        algorithms::approx::{Approx, Tolerance},
        objects::{Cycle, Face},
        operations::{BuildCycle, BuildFace, Insert, UpdateFace, UpdateRegion},
        services::Services,
    };

    use super::Triangulate;

    #[test]
    fn simple() -> anyhow::Result<()> {
        let mut services = Services::new();

        let a = [0., 0.];
        let b = [2., 0.];
        let c = [2., 2.];
        let d = [0., 1.];

        let face =
            Face::unbound(services.objects.surfaces.xy_plane(), &mut services)
                .update_region(|region| {
                    region
                        .update_exterior(|_| {
                            Cycle::polygon([a, b, c, d], &mut services)
                                .insert(&mut services)
                        })
                        .insert(&mut services)
                });
        services.only_validate(&face);

        let a = Point::from(a).to_xyz();
        let b = Point::from(b).to_xyz();
        let c = Point::from(c).to_xyz();
        let d = Point::from(d).to_xyz();

        let triangles = triangulate(face)?;

        assert!(triangles.contains_triangle([a, b, d]));
        assert!(triangles.contains_triangle([b, c, d]));
        assert!(!triangles.contains_triangle([a, b, c]));
        assert!(!triangles.contains_triangle([a, c, d]));

        Ok(())
    }

    #[test]
    fn simple_hole() -> anyhow::Result<()> {
        let mut services = Services::new();

        let a = [0., 0.];
        let b = [4., 0.];
        let c = [4., 4.];
        let d = [0., 4.];

        let e = [1., 1.];
        let f = [1., 2.];
        let g = [3., 3.];
        let h = [3., 1.];

        let surface = services.objects.surfaces.xy_plane();

        let face = Face::unbound(surface.clone(), &mut services).update_region(
            |region| {
                region
                    .update_exterior(|_| {
                        Cycle::polygon([a, b, c, d], &mut services)
                            .insert(&mut services)
                    })
                    .add_interiors([Cycle::polygon(
                        [e, f, g, h],
                        &mut services,
                    )
                    .insert(&mut services)])
                    .insert(&mut services)
            },
        );
        services.only_validate(&face);

        let triangles = triangulate(face)?;

        let a = surface.geometry().point_from_surface_coords(a);
        let b = surface.geometry().point_from_surface_coords(b);
        let e = surface.geometry().point_from_surface_coords(e);
        let f = surface.geometry().point_from_surface_coords(f);
        let g = surface.geometry().point_from_surface_coords(g);
        let h = surface.geometry().point_from_surface_coords(h);

        // Let's test that some correct triangles are present. We don't need to
        // test them all.
        //
        // Please note that there are multiple valid triangulations of any given
        // polygon. So if you change the input data above, or the algorithm, the
        // following assertions might break.
        //
        // This limits the usefulness of this test. It would be better to have a
        // smarter way of verifying the triangulation.
        assert!(triangles.contains_triangle([a, b, e]));
        assert!(triangles.contains_triangle([b, e, h]));

        // Shouldn't contain any possible triangle from the hole.
        assert!(!triangles.contains_triangle([e, f, g]));
        assert!(!triangles.contains_triangle([e, g, h]));
        assert!(!triangles.contains_triangle([e, f, h]));
        assert!(!triangles.contains_triangle([f, g, h]));

        Ok(())
    }

    #[test]
    fn sharp_concave_shape() -> anyhow::Result<()> {
        let mut services = Services::new();

        //   e       c
        //   |\     /|
        //   \ \   / b
        //    \ \ / /
        //     \ d /
        //      \a/

        // Naive Delaunay triangulation will create a triangle (c, d, e), which
        // is not part of the polygon. The higher-level triangulation will
        // filter that out, but it will result in missing triangles.

        let a = [1., 0.];
        let b = [2., 8.];
        let c = [2., 9.];
        let d = [1., 1.];
        let e = [0., 9.];

        let surface = services.objects.surfaces.xy_plane();

        let face = Face::unbound(surface.clone(), &mut services).update_region(
            |region| {
                region
                    .update_exterior(|_| {
                        Cycle::polygon([a, b, c, d, e], &mut services)
                            .insert(&mut services)
                    })
                    .insert(&mut services)
            },
        );
        services.only_validate(&face);

        let triangles = triangulate(face)?;

        let a = surface.geometry().point_from_surface_coords(a);
        let b = surface.geometry().point_from_surface_coords(b);
        let c = surface.geometry().point_from_surface_coords(c);
        let d = surface.geometry().point_from_surface_coords(d);
        let e = surface.geometry().point_from_surface_coords(e);

        assert!(triangles.contains_triangle([a, b, d]));
        assert!(triangles.contains_triangle([a, d, e]));
        assert!(triangles.contains_triangle([b, c, d]));

        Ok(())
    }

    fn triangulate(face: Face) -> anyhow::Result<Mesh<Point<3>>> {
        let tolerance = Tolerance::from_scalar(Scalar::ONE)?;
        Ok(face.approx(tolerance).triangulate())
    }
}
