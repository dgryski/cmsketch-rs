const BITS_PER_WORD: usize = 64;

use crate::hash::{jenkins_hash, leveldb_bloom_hash};

struct BitVector(Vec<u64>);

impl BitVector {
    fn new(w: usize) -> BitVector {
        let blen = (w + BITS_PER_WORD - 1) / BITS_PER_WORD;
        let mut bits = Vec::<u64>::new();
        bits.resize(blen, 0);
        BitVector(bits)
    }

    fn test(&self, bit: usize) -> u64 {
        self.0[bit / BITS_PER_WORD] & (1 << (bit % BITS_PER_WORD))
    }

    fn set(&mut self, bit: usize) {
        self.0[bit / BITS_PER_WORD] |= 1 << (bit % BITS_PER_WORD)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct BFilter {
    width: u32,
    hashes: u32,
    bits: BitVector,
}

pub enum MergeError {
    DimensionMismatch,
}

impl BFilter {
    pub fn new(width: u32, hashes: u32) -> BFilter {
        let bits = BitVector::new(width as usize);
        BFilter {
            width,
            hashes,
            bits,
        }
    }

    pub fn add(&mut self, b: &[u8]) {
        let h1 = leveldb_bloom_hash(b);
        let h2 = jenkins_hash(b);

        for i in 0..self.hashes {
            let pos = h1.wrapping_add(h2.wrapping_mul(i)) as usize % (self.width as usize);
            self.bits.set(pos)
        }
    }

    pub fn exists(&self, b: &[u8]) -> bool {
        let h1 = leveldb_bloom_hash(b);
        let h2 = jenkins_hash(b);

        for i in 0..self.hashes {
            let pos = h1.wrapping_add(h2.wrapping_mul(i)) as usize % (self.width as usize);
            if self.bits.test(pos) == 0 {
                return false;
            }
        }
        true
    }

    pub fn merge(&mut self, other: &BFilter) -> Result<(), MergeError> {
        if self.bits.len() != other.bits.len() {
            return Err(MergeError::DimensionMismatch);
        }

        for (i, w) in self.bits.0.iter_mut().enumerate() {
            *w |= other.bits.0[i]
        }

        Ok(())
    }

    pub fn compress(&mut self) {
        let mut bits = BitVector::new((self.width as usize) / 2);

        self.width /= 2;
        let blen = bits.0.len();

        for idx in 0..blen as usize {
            bits.0[idx] = self.bits.0[idx] | self.bits.0[idx + blen as usize];
        }

        self.bits = bits;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_bfilter() {
        let mut bf = BFilter::new(1024, 8);

        let words = ["foo", "bar", "baz", "qux", "zot", "frob", "zork", "zeek"];

        for &w in words.iter() {
            bf.add(w.as_bytes());
        }

        for &w in words.iter() {
            assert!(bf.exists(w.as_bytes()));
        }

        assert!(!bf.exists("hello, world".as_bytes()));

        bf.compress();

        for &w in words.iter() {
            assert!(bf.exists(w.as_bytes()));
        }
    }

    #[test]
    fn test_merge() {
        let mut bf0 = BFilter::new(1024, 8);
        let mut bf1 = BFilter::new(1024, 8);

        let words0 = ["foo", "bar", "baz", "qux"];
        let words1 = ["zot", "frob", "zork", "zeek"];

        for &w in words0.iter() {
            bf0.add(w.as_bytes());
        }

        for &w in words1.iter() {
            bf1.add(w.as_bytes());
        }

        let result = bf0.merge(&bf1);
        assert!(result.is_ok());

        for &w in words0.iter() {
            assert!(bf0.exists(w.as_bytes()));
        }

        for &w in words1.iter() {
            assert!(bf0.exists(w.as_bytes()));
        }
    }
}
