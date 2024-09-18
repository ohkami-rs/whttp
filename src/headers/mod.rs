mod hash;
mod name;
mod value;

pub use name::{Header, standard};
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

    impl PartialEq for Headers {
        fn eq(&self, other: &Self) -> bool {
            for (h, v) in self.iter() {
                if other.get(h) != Some(v) {
                    return false
                }
            }
            true
        }
    }

    impl std::ops::Index<Header> for Headers {
        type Output = str;

        #[inline]
        fn index(&self, header: Header) -> &Self::Output {
            self.get(header).unwrap_or_default()
        }
    }
};

#[inline(always)]
const fn hasher((h, _): &(Header, Value)) -> u64 {
    h.hash
}

#[inline(always)]
const fn eq_to(header: &Header) -> impl Fn(&(Header, Value)) -> bool + '_ {
    |(h, _)| h.hash == header.hash
}

impl Headers {
    pub fn new() -> Self {
        // 8 is elected heuristically
        Self { table: RawTable::with_capacity(8) }
    }

    #[inline]
    pub fn get(&self, header: Header) -> Option<&str> {
        match self.table.get(header.hash, eq_to(&header)) {
            Some((_, v)) => Some(&*v),
            None => None
        }
    }

    #[inline]
    pub fn insert(&mut self, header: Header, value: impl Into<Value>) {
        let value = value.into();
        match self.table.find_or_find_insert_slot(header.hash as u64, eq_to(&header), hasher) {
            Err(slot)  => unsafe {self.table.insert_in_slot(header.hash, slot, (header, value));}
            Ok(bucket) => unsafe {bucket.as_mut().1 = value}
        }
    }

    #[inline]
    pub fn append(&mut self, header: Header, value: impl Into<Value>) {
        let value = value.into();
        match self.table.find_or_find_insert_slot(header.hash as u64, eq_to(&header), hasher) {
            Err(slot)  => unsafe {self.table.insert_in_slot(header.hash, slot, (header, value));}
            Ok(bucket) => unsafe {bucket.as_mut().1.append(value)}
        }
    }

    #[inline]
    pub fn remove(&mut self, header: Header) {
        self.table.remove_entry(header.hash, eq_to(&header));
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (Header, &str)> {
        // SAFETY: lifetime of `&self` and `&str` is the same
        unsafe {self.table.iter()}.map(|bucket| {
            let (h, v) = unsafe {bucket.as_ref()};
            (*h, &**v)
        })
    }

    #[inline]
    pub fn clear(&mut self) {
        self.table.clear()
    }
}
