use std::any::{Any, TypeId};

use hashbrown::HashMap;

#[derive(Default)]
pub struct TypeMap {
    inner: HashMap<TypeId, Box<dyn Any + Send + Sync>>
}

impl TypeMap where {
    #[inline]
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    #[inline]
    pub fn get<K: 'static>(&self) -> Option<&K> {
        self.inner.get(&TypeId::of::<K>()).map(|f| unsafe { f.downcast_ref_unchecked() })
    }

    #[inline]
    pub fn get_mut<K: 'static>(&mut self) -> Option<&mut K> {
        self.inner.get_mut(&TypeId::of::<K>()).map(|f| unsafe { f.downcast_mut_unchecked() })
    }

    #[inline]
    pub fn insert<K: 'static + Send + Sync>(&mut self, value: K) {
        self.inner.insert(TypeId::of::<K>(), Box::new(value));
    }
}
