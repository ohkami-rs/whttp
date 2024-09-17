mod hash;
mod name;

pub use name::{Header, standard::*};

use std::ptr::NonNull;
use ::hashbrown::raw::RawTable;

pub struct Headers {
    table: RawTable<(Header, NonNull<str>)>
}

impl Headers {
    pub fn new() -> Self {
        // 8 is elected heuristically
        Self { table: RawTable::with_capacity(8) }
    }

    #[inline]
    pub fn insert(&mut self, header: Header, value: )
}
