//! case-ignoring fxhash

use super::name::{normalized, InvalidHeader};
use std::{hash::Hasher, ops::BitXor};

#[cfg(target_pointer_width = "32")]
const K: usize = 0x9e3779b9;
#[cfg(target_pointer_width = "64")]
const K: usize = 0x517cc1b727220a95;

#[inline(always)]
fn take_first_chunk<'s, const N: usize>(slice: &mut &'s [u8]) -> Option<&'s [u8; N]> {
    let (first, tail) = slice.split_first_chunk()?;
    *slice = tail;
    Some(first)
}

#[derive(Clone)]
pub struct HeaderHasher {
    hash: usize,
}

impl HeaderHasher {
    #[inline(always)]
    fn add(&mut self, word: usize) {
        self.hash = self.hash.rotate_left(5).bitxor(word).wrapping_mul(K);
    }
}

impl Default for HeaderHasher {
    #[inline(always)]
    fn default() -> Self {
        Self { hash: 0 }
    }
}

impl Hasher for HeaderHasher {
    #[inline]
    fn write(&mut self, mut bytes: &[u8]) {
        let mut state = self.clone();

        while let Some(&[a, b, c, d, e, f, g, h]) = take_first_chunk(&mut bytes) {
            state.add(usize::from_ne_bytes([
                normalized(a).unwrap(),
                normalized(b).unwrap(),
                normalized(c).unwrap(),
                normalized(d).unwrap(),
                normalized(e).unwrap(),
                normalized(f).unwrap(),
                normalized(g).unwrap(),
                normalized(h).unwrap(),
            ]));
        }
        if let Some(&[a, b, c, d]) = take_first_chunk(&mut bytes) {
            state.add(u32::from_ne_bytes([
                normalized(a).unwrap(),
                normalized(b).unwrap(),
                normalized(c).unwrap(),
                normalized(d).unwrap(),
            ]) as usize);
        }
        if let Some(&[a, b]) = take_first_chunk(&mut bytes) {
            state.add(u16::from_ne_bytes([
                normalized(a).unwrap(),
                normalized(b).unwrap(),
            ]) as usize);
        }
        if let Some(&[a]) = take_first_chunk(&mut bytes) {
            state.add(normalized(a).unwrap() as usize);
        }

        *self = state;
    }

    #[inline(always)]
    fn finish(&self) -> u64 {
        self.hash as _
    }
}

#[inline(always)]
pub(crate) fn normalized_hash(bytes: &[u8]) -> Result<usize, InvalidHeader> {
    let mut h = HeaderHasher::default();
    h.write(bytes);
    Ok(h.hash)
}

pub(crate) const fn const_normalized_hash(mut bytes: &[u8]) -> Result<usize, InvalidHeader> {
    let mut hash: usize = 0;

    while let Some((&[a, b, c, d, e, f, g, h], rest)) = bytes.split_first_chunk() {
        hash = const_next_hash(hash, usize::from_ne_bytes([
            match normalized(a) {Ok(a) => a, Err(err) => return Err(err)},
            match normalized(b) {Ok(b) => b, Err(err) => return Err(err)},
            match normalized(c) {Ok(c) => c, Err(err) => return Err(err)},
            match normalized(d) {Ok(d) => d, Err(err) => return Err(err)},
            match normalized(e) {Ok(e) => e, Err(err) => return Err(err)},
            match normalized(f) {Ok(f) => f, Err(err) => return Err(err)},
            match normalized(g) {Ok(g) => g, Err(err) => return Err(err)},
            match normalized(h) {Ok(h) => h, Err(err) => return Err(err)},
        ]));
        bytes = rest
    }
    if let Some((&[a, b, c, d], rest)) = bytes.split_first_chunk() {
        hash = const_next_hash(hash, u32::from_ne_bytes([
            match normalized(a) {Ok(a) => a, Err(err) => return Err(err)},
            match normalized(b) {Ok(b) => b, Err(err) => return Err(err)},
            match normalized(c) {Ok(c) => c, Err(err) => return Err(err)},
            match normalized(d) {Ok(d) => d, Err(err) => return Err(err)},
        ]) as usize);
        bytes = rest
    }
    if let Some((&[a, b], rest)) = bytes.split_first_chunk() {
        hash = const_next_hash(hash, u16::from_ne_bytes([
            match normalized(a) {Ok(a) => a, Err(err) => return Err(err)},
            match normalized(b) {Ok(b) => b, Err(err) => return Err(err)},
        ]) as usize);
        bytes = rest
    }
    if let Some((&[a], _)) = bytes.split_first_chunk() {
        hash = const_next_hash(hash, match normalized(a) {Ok(a) => a, Err(err) => return Err(err)} as usize);
    }

    Ok(hash)
}

const fn const_next_hash(current: usize, word: usize) -> usize {
    let mut hash = current.rotate_left(5);
    hash = const_xor(hash, word);
    hash.wrapping_mul(K)
}

const fn const_xor(a: usize, b: usize) -> usize {
    const fn byte_xor(a: u8, b: u8) -> u8 {
        let mut result = 0;
        if (a & 0b1) != (b & 0b1) {result += 0b1}
        if (a & 0b10) != (b & 0b10) {result += 0b10}
        if (a & 0b100) != (b & 0b100) {result += 0b100}
        if (a & 0b1000) != (b & 0b1000) {result += 0b1000}
        if (a & 0b10000) != (b & 0b10000) {result += 0b10000}
        if (a & 0b100000) != (b & 0b100000) {result += 0b100000}
        if (a & 0b1000000) != (b & 0b1000000) {result += 0b1000000}
        if (a & 0b10000000) != (b & 0b10000000) {result += 0b10000000}
        result
    }

    let mut bytes = a.to_ne_bytes();
    let b_bytes = b.to_ne_bytes();
    let mut i = 0;
    while i < std::mem::size_of::<usize>() {
        bytes[i] = byte_xor(bytes[i], b_bytes[i]);
        i += 1
    }
    usize::from_ne_bytes(bytes)
}

#[cfg(test)]
#[test]
fn test_xor() {
    for (a, b, expected) in CASES {
        assert_eq!(const_xor(*a, *b), *expected)
    }

    const CASES: &'static [(usize, usize, usize)] = &[
        (12, 23, 27),
        (23, 34, 53),
        (34, 45, 15),
        (45, 56, 21),
        (56, 67, 123),
        (67, 78, 13),
        (78, 89, 23),
        (89, 90, 3),
    ];
}

