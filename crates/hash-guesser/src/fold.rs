//! FNV-1a32, forwards and backwards.
//!
//! Kept free of every other module so the integration tests can include it
//! directly, the same way `tests/splitter.rs` includes `words.rs`.

pub const FNV_OFFSET: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;
/// `FNV_PRIME^-1 mod 2^32`. The prime is odd, so it is a unit mod 2^32 and the
/// multiply step is invertible - which is what makes `fnv1a_back` possible.
const FNV_PRIME_INV: u32 = 0x359c_449b;

/// FNV-1a over the lowercased bytes, continued from `h`.
///
/// Continuable because that is the whole performance story: a name is built one
/// word at a time, and every prefix of it has already been hashed. Rehashing
/// the full string at each step would make the force mode quadratic in name
/// length for no reason.
#[inline]
pub fn fnv1a_cont(data: &[u8], mut h: u32) -> u32 {
    for &c in data {
        let c = if c.is_ascii_uppercase() { c + 32 } else { c };
        h ^= c as u32;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

#[inline]
pub fn fnv1a(name: &str) -> u32 {
    fnv1a_cont(name.as_bytes(), FNV_OFFSET)
}

/// The inverse of `fnv1a_cont`: given `h == fnv1a_cont(data, x)`, return `x`.
///
/// This is what makes suffix anchoring free. FNV folds left to right, so a
/// *prefix* is hashed once and reused - but a suffix would otherwise have to be
/// re-hashed on every single probe. Running the fold backwards from a target
/// instead answers "what state would the search have to reach for this target to
/// end in this suffix?", once per target, before the search starts. The hot loop
/// then never sees the suffix at all.
///
/// Lowercases exactly as the forward fold does, so `--suffix data` and
/// `--suffix Data` fold to the same state.
#[inline]
pub fn fnv1a_back(data: &[u8], mut h: u32) -> u32 {
    for &c in data.iter().rev() {
        let c = if c.is_ascii_uppercase() { c + 32 } else { c };
        h = h.wrapping_mul(FNV_PRIME_INV) ^ c as u32;
    }
    h
}
