mod hash;
mod name;
mod value;

pub use name::{Header, standard};
pub use value::Value;

use ::hashbrown::hash_table::{HashTable, Entry};

pub struct Headers {
    table: HashTable<(Header, Value)>
}

const _/* trait impls */: () = {
    impl Default for Headers {
        fn default() -> Self {
            Self::new()
        }
    }

    impl std::fmt::Debug for Headers {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_map()
                .entries(self.iter())
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

    impl std::ops::Index<&Header> for Headers {
        type Output = str;

        #[inline]
        fn index(&self, header: &Header) -> &Self::Output {
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
    pub const fn new() -> Self {
        Self { table: HashTable::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { table: HashTable::with_capacity(capacity) }
    }

    #[inline]
    pub fn get(&self, header: &Header) -> Option<&str> {
        match self.table.find(header.hash, eq_to(header)) {
            Some((_, v)) => Some(&*v),
            None => None
        }
    }

    #[inline]
    pub fn insert(&mut self, header: &Header, value: impl Into<Value>) {
        let value = value.into();
        match self.table.entry(header.hash, eq_to(header), hasher) {
            Entry::Occupied(mut entry) => {entry.get_mut().1 = value;}
            Entry::Vacant(entry) => {entry.insert((header.clone(), value));}
        }
    }

    #[inline]
    pub fn remove(&mut self, header: &Header) -> Option<Value> {
        if let Ok(entry) = self.table.find_entry(header.hash, eq_to(header)) {
            Some(entry.remove().0.1)
        } else {
            None
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.table.clear()
    }

    #[inline]
    pub fn append(&mut self, header: &Header, value: impl Into<Value>) -> &mut Self {
        let value = value.into();
        match self.table.entry(header.hash, eq_to(header), hasher) {
            Entry::Occupied(mut entry) => {entry.get_mut().1.append(value);}
            Entry::Vacant(entry) => {entry.insert((header.clone(), value));}
        }
        self
    }

    /// append with owned `Header` (mainly used in request parsing)
    #[inline]
    pub fn push(&mut self, header: Header, value: impl Into<Value>) -> &mut Self {
        let value = value.into();
        match self.table.entry(header.hash, eq_to(&header), hasher) {
            Entry::Occupied(mut entry) => {entry.get_mut().1.append(value);}
            Entry::Vacant(entry) => {entry.insert((header, value));}
        }
        self
    }

    #[inline]
    pub fn set(&mut self, header: &Header, setter: impl SetHeader) -> &mut Self {
        setter.set(header, self);
        self
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&Header, &str)> {
        self.table.iter().map(|(h, v)| (h, &**v))
    }
}

pub trait SetHeader {
    fn set(self, header: &Header, headers: &mut Headers);
}
const _: () = {
    impl SetHeader for Option<()> {
        #[inline]
        fn set(self, header: &Header, headers: &mut Headers) {
            headers.remove(header);
        }
    }
    impl<V: Into<Value>> SetHeader for V {
        #[inline]
        fn set(self, header: &Header, headers: &mut Headers) {
            headers.insert(header, self.into());
        }
    }
};
