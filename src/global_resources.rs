use std::sync::Arc;

pub trait GlobalResource = Sync + Send + 'static;

pub struct GlobalResources(Arc<crate::typemap::TypeMap>);

impl GlobalResources {
    pub fn get<R: GlobalResource>(&self) -> Option<&R> {
        self.0.get::<R>()
    }
}

#[derive(Default)]
pub struct GlobalResourcesBuilder(crate::typemap::TypeMap);

impl GlobalResourcesBuilder {
    pub fn register_resource<R: GlobalResource>(mut self, resource: R) -> Self {
        self.0.insert(resource);
        self
    }

    // pub fn register_resource_type<R: GlobalResource>(mut self) -> Self {
    //     self
    // }

    pub fn build(self) -> GlobalResources {
        GlobalResources(Arc::new(self.0))
    }
}
