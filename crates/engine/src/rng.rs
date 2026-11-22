//! Générateur pseudo-aléatoire déterministe (SplitMix64).
//!
//! Identique sur toutes les plateformes, sérialisable : c'est le seul
//! aléatoire autorisé dans le moteur et la génération de monde.

use serde::{Deserialize, Serialize};

const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { state: mix64(seed ^ GOLDEN) }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN);
        mix64(self.state)
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Entier uniforme dans `0..n` (`n > 0`).
    #[inline]
    pub fn below(&mut self, n: u32) -> u32 {
        debug_assert!(n > 0);
        ((self.next_u32() as u64 * n as u64) >> 32) as u32
    }

    /// Entier uniforme dans `lo..=hi`.
    pub fn range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        debug_assert!(lo <= hi);
        lo + self.below((hi - lo + 1) as u32) as i32
    }

    /// Vrai avec une probabilité de `p` pour mille.
    #[inline]
    pub fn chance_permille(&mut self, p: u32) -> bool {
        self.below(1000) < p
    }

    /// Flottant uniforme dans `[0, 1)`. Réservé à la génération de monde :
    /// la simulation n'utilise jamais de flottants.
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    pub fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }
}

/// Mélangeur de bits (finaliseur SplitMix64).
#[inline]
pub fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Hachage sans état de deux valeurs, utile pour du bruit reproductible.
#[inline]
pub fn hash2(a: u64, b: u64) -> u64 {
    mix64(a ^ mix64(b.wrapping_add(GOLDEN)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = Rng::new(7);
        for n in 1..200 {
            assert!(rng.below(n) < n);
        }
    }
}
