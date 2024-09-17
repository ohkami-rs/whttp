use std::{borrow::Cow, collections::HashMap, hash::BuildHasherDefault};

mod hash;
mod name;

pub use name::{Header, standard::*};

pub struct Headers(HashMap<
    Header,
    Cow<'static, str>,
    BuildHasherDefault<hash::HeaderHasher>>
);


