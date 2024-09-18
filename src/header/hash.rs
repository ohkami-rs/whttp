//! based on
//! [rustc-hash v2](https://github.com/rust-lang/rustc-hash/blob/eb049a8209f58003957c34477a2d8d2729f6b633/src/lib.rs)
//! ; MIT

use super::name::{normalized, InvalidHeader};

#[cfg(target_pointer_width = "64")]
const K: usize = 0xf1357aea2e62a9c5;
#[cfg(target_pointer_width = "32")]
const K: usize = 0x93d765dd;

// Nothing special, digits of pi.
const SEED1: u64 = 0x243f6a8885a308d3;
const SEED2: u64 = 0x13198a2e03707344;
const PREVENT_TRIVIAL_ZERO_COLLAPSE: u64 = 0xa4093822299f31d0;

macro_rules! r#try {
    ($result:expr) => {
        match $result {
            Ok(ok) => ok,
            Err(e) => return Err(e)
        }
    };
}

#[inline]
pub(crate) const fn normalized_hash(bytes: &[u8]) -> Result<u64, InvalidHeader> {
    #[inline]
    const fn next(current: usize, word: usize) -> usize {
        current.wrapping_add(word).wrapping_mul(K)
    }

    let word = r#try!(normalized_hash_core(bytes));

    #[allow(unused_mut)]
    let mut hash = next(0, word as usize);
    #[cfg(target_pointer_width = "32")] {
        hash = next(hash, (word >> 32) as usize);
    }

    Ok(hash as u64)
}

#[inline]
const fn multiply_mix(x: u64, y: u64) -> u64 {
    #[cfg(target_pointer_width = "64")]
    {
        // We compute the full u64 x u64 -> u128 product, this is a single mul
        // instruction on x86-64, one mul plus one mulhi on ARM64.
        let full = (x as u128) * (y as u128);
        let lo = full as u64;
        let hi = (full >> 64) as u64;

        // The middle bits of the full product fluctuate the most with small
        // changes in the input. This is the top bits of lo and the bottom bits
        // of hi. We can thus make the entire output fluctuate with small
        // changes to the input by XOR'ing these two halves.
        lo ^ hi

        // Unfortunately both 2^64 + 1 and 2^64 - 1 have small prime factors,
        // otherwise combining with + or - could result in a really strong hash, as:
        //     x * y = 2^64 * hi + lo = (-1) * hi + lo = lo - hi,   (mod 2^64 + 1)
        //     x * y = 2^64 * hi + lo =    1 * hi + lo = lo + hi,   (mod 2^64 - 1)
        // Multiplicative hashing is universal in a field (like mod p).
    }

    #[cfg(target_pointer_width = "32")]
    {
        // u64 x u64 -> u128 product is prohibitively expensive on 32-bit.
        // Decompose into 32-bit parts.
        let lx = x as u32;
        let ly = y as u32;
        let hx = (x >> 32) as u32;
        let hy = (y >> 32) as u32;

        // u32 x u32 -> u64 the low bits of one with the high bits of the other.
        let afull = (lx as u64) * (hy as u64);
        let bfull = (hx as u64) * (ly as u64);

        // Combine, swapping low/high of one of them so the upper bits of the
        // product of one combine with the lower bits of the other.
        afull ^ bfull.rotate_right(32)
    }
}

#[inline]
const fn normalized_hash_core(bytes: &[u8]) -> Result<u64, InvalidHeader> {
    
    #[inline(always)]
    const unsafe fn normalized_array_from_slice_and_start<const N: usize>(
        slice: &[u8],
        start: usize
    ) -> Result<[u8; N], InvalidHeader> {
        use std::mem::MaybeUninit;

        #[cfg(debug_assertions)] {
            if slice.len() - start < N {panic!("invalid slice len and start")}
        }

        let mut a = [const {MaybeUninit::uninit()}; N];
        let mut i = 0;
        while i < N {
            a[i] = MaybeUninit::new(r#try!(normalized(slice[start + i])));
            i += 1
        }

        let ptr = &a as *const [MaybeUninit<u8>; N] as *const [u8; N];
        Ok(unsafe {*ptr})
    }

    //////////////////////////////////////////////////////////////////////////////////////////

    let len = bytes.len();
    let mut s0 = SEED1;
    let mut s1 = SEED2;

    // SAFETY: len checks

    if len <= 16 {
        // XOR the input into s0, s1.
        if len >= 8 {
            s0 ^= u64::from_le_bytes(r#try!(unsafe {normalized_array_from_slice_and_start(bytes, 0)}));
            s1 ^= u64::from_le_bytes(r#try!(unsafe {normalized_array_from_slice_and_start(bytes, len - 8)}));
        } else if len >= 4 {
            s0 ^= u32::from_le_bytes(r#try!(unsafe {normalized_array_from_slice_and_start(bytes, 0)})) as u64;
            s1 ^= u32::from_le_bytes(r#try!(unsafe {normalized_array_from_slice_and_start(bytes, len - 4)})) as u64;
        } else if len > 0 {
            let lo = r#try!(normalized(bytes[0]));
            let mid = r#try!(normalized(bytes[len / 2]));
            let hi = r#try!(normalized(bytes[len - 1]));
            s0 ^= lo as u64;
            s1 ^= ((hi as u64) << 8) | mid as u64;
        }
    } else {
        // Handle bulk (can partially overlap with suffix).
        let mut off = 0;
        while off < len - 16 {
            let x = u64::from_le_bytes(r#try!(unsafe {normalized_array_from_slice_and_start(bytes, off)}));
            let y = u64::from_le_bytes(r#try!(unsafe {normalized_array_from_slice_and_start(bytes, off + 8)}));

            // Replace s1 with a mix of s0, x, and y, and s0 with s1.
            // This ensures the compiler can unroll this loop into two
            // independent streams, one operating on s0, the other on s1.
            //
            // Since zeroes are a common input we prevent an immediate trivial
            // collapse of the hash function by XOR'ing a constant with y.
            let t = multiply_mix(s0 ^ x, PREVENT_TRIVIAL_ZERO_COLLAPSE ^ y);
            s0 = s1;
            s1 = t;
            off += 16;
        }

        s0 ^= u64::from_le_bytes(r#try!(unsafe {normalized_array_from_slice_and_start(bytes, len - 16)}));
        s1 ^= u64::from_le_bytes(r#try!(unsafe {normalized_array_from_slice_and_start(bytes, len - 8)}));
    }

    Ok(multiply_mix(s0, s1) ^ (len as u64))
}

#[cfg(test)]
pub fn original_hash_bytes(bytes: &[u8]) -> u64 {
    let len = bytes.len();
    let mut s0 = SEED1;
    let mut s1 = SEED2;

    if len <= 16 {
        // XOR the input into s0, s1.
        if len >= 8 {
            s0 ^= u64::from_le_bytes(bytes[0..8].try_into().unwrap());
            s1 ^= u64::from_le_bytes(bytes[len - 8..].try_into().unwrap());
        } else if len >= 4 {
            s0 ^= u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as u64;
            s1 ^= u32::from_le_bytes(bytes[len - 4..].try_into().unwrap()) as u64;
        } else if len > 0 {
            let lo = bytes[0];
            let mid = bytes[len / 2];
            let hi = bytes[len - 1];
            s0 ^= lo as u64;
            s1 ^= ((hi as u64) << 8) | mid as u64;
        }
    } else {
        // Handle bulk (can partially overlap with suffix).
        let mut off = 0;
        while off < len - 16 {
            let x = u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap());
            let y = u64::from_le_bytes(bytes[off + 8..off + 16].try_into().unwrap());

            // Replace s1 with a mix of s0, x, and y, and s0 with s1.
            // This ensures the compiler can unroll this loop into two
            // independent streams, one operating on s0, the other on s1.
            //
            // Since zeroes are a common input we prevent an immediate trivial
            // collapse of the hash function by XOR'ing a constant with y.
            let t = multiply_mix(s0 ^ x, PREVENT_TRIVIAL_ZERO_COLLAPSE ^ y);
            s0 = s1;
            s1 = t;
            off += 16;
        }

        let suffix = &bytes[len - 16..];
        s0 ^= u64::from_le_bytes(suffix[0..8].try_into().unwrap());
        s1 ^= u64::from_le_bytes(suffix[8..16].try_into().unwrap());
    }

    multiply_mix(s0, s1) ^ (len as u64)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_ignoring() {
        //assert_eq!(n);
    }
}
