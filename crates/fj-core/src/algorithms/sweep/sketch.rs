use fj_math::Vector;

use crate::{
    objects::{Sketch, Solid, Surface},
    operations::Insert,
    services::Services,
    storage::Handle,
};

use super::{Sweep, SweepCache};

impl Sweep for (Handle<Sketch>, Handle<Surface>) {
    type Swept = Handle<Solid>;

    fn sweep_with_cache(
        self,
        path: impl Into<Vector<3>>,
        cache: &mut SweepCache,
        services: &mut Services,
    ) -> Self::Swept {
        let path = path.into();

        let mut shells = Vec::new();
        for face in self.0.faces(self.1, services) {
            let shell = face.sweep_with_cache(path, cache, services);
            shells.push(shell);
        }

        Solid::new(shells).insert(services)
    }
}
