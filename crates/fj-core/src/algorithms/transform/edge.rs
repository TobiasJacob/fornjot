use fj_math::Transform;

use crate::{
    objects::{GlobalEdge, HalfEdge},
    services::Services,
};

use super::{TransformCache, TransformObject};

impl TransformObject for HalfEdge {
    fn transform_with_cache(
        self,
        transform: &Transform,
        services: &mut Services,
        cache: &mut TransformCache,
    ) -> Self {
        // Don't need to transform the path, as that's defined in surface
        // coordinates.
        let path = self.path();
        let boundary = self.boundary();
        let curve = self
            .curve()
            .clone()
            .transform_with_cache(transform, services, cache);
        let start_vertex = self
            .start_vertex()
            .clone()
            .transform_with_cache(transform, services, cache);
        let global_form = self
            .global_form()
            .clone()
            .transform_with_cache(transform, services, cache);

        Self::new(path, boundary, curve, start_vertex, global_form)
    }
}

impl TransformObject for GlobalEdge {
    fn transform_with_cache(
        self,
        _: &Transform,
        _: &mut Services,
        _: &mut TransformCache,
    ) -> Self {
        // There's nothing to actually transform here, as `GlobalEdge` holds no
        // data. We still need this implementation though, as a new `GlobalEdge`
        // object must be created to represent the new and transformed edge.
        Self::new()
    }
}
