mod hash;
mod name;
mod value;

pub use name::{Header, standard::*};
pub use value::Value;

use ::hashbrown::raw::RawTable;

pub struct Headers {
    table: RawTable<(Header, Value)>
}

const _/* trait impls */: () = {
    impl std::fmt::Debug for Headers {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // SAFETY: `self.table` outlives `table_iter`
            let table_iter = unsafe {self.table.iter()};
            f.debug_map()
                .entries(table_iter.map(|bucket| unsafe {bucket.as_ref()}.clone()))
                .finish()
        }
    }
};

#[inline(always)]
const fn hasher((h, _): &(Header, Value)) -> u64 {
    h.hash
}

#[inline(always)]
const fn eq_to(header: &Header) -> impl Fn(&(Header, Value)) -> bool+'_ {
    |(h, _)| h.hash == header.hash
}

impl Headers {
    pub fn new() -> Self {
        // 8 is elected heuristically
        Self { table: RawTable::with_capacity(8) }
    }

    #[inline]
    pub fn insert(&mut self, header: Header, value: impl Into<Value>) {
        let value = value.into();
        match self.table.find_or_find_insert_slot(header.hash as u64, eq_to(&header), hasher) {
            Ok(bucket) => unsafe {bucket.as_mut().1 = value}
            Err(slot)  => unsafe {self.table.insert_in_slot(header.hash, slot, (header, value));}
        }
    }
}
