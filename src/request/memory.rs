use core::any::{Any, TypeId};
use core::hash::{BuildHasherDefault, Hasher};
use ::hashbrown::HashMap;

pub(super) struct Memory(Option<Box<
    HashMap<
        TypeId,
        Box<dyn Any + Send + Sync>,
        BuildHasherDefault<TypeIdHasher>
    >
>>);

/// prevent redundant calculation from the typeid value itself
#[derive(Default)]
struct TypeIdHasher(u64);
impl Hasher for TypeIdHasher {
    #[cold]
    fn write(&mut self, _: &[u8]) {
        unreachable!()
    }

    #[inline]
    fn write_u64(&mut self, typeid_value: u64) {
        self.0 = typeid_value
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0    
    }
}

impl Memory {
    pub(super) const fn new() -> Self {
        Self(None)
    }

    pub(super) fn clear(&mut self) {
        if let Some(this) = &mut self.0 {
            this.clear();
        }
    }

    pub(super) fn insert<Data: Send + Sync + 'static>(&mut self, data: Data) {
        self.0.get_or_insert_with(|| Box::new(HashMap::default()))
            .insert(TypeId::of::<Data>(), Box::new(data));
    }

    pub(super) fn get<Data: Send + Sync + 'static>(&self) -> Option<&Data> {
        self.0.as_ref()?
            .get(&TypeId::of::<Data>())
            .map(|box_any| {
                let any: &dyn Any = &**box_any;
                #[cfg(debug_assertions)] {
                    assert!(any.is::<Data>(), "Memory is poisoned!!!")
                }
                unsafe {&*(any as *const dyn Any as *const Data)}
            })
    }
}
