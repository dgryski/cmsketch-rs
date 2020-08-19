use crate::hash::{jenkins_hash, leveldb_bloom_hash};

pub struct Sketch {
    width: u32,
    depth: u32,
    sk: Vec<Vec<u32>>,
}

pub enum MergeError {
    DimensionMismatch,
}

impl Sketch {
    pub fn new(width: u32, depth: u32) -> Sketch {
        let mut sk = Vec::<Vec<u32>>::new();
        sk.reserve(depth as usize);

        for _ in 0..depth {
            let mut row = Vec::<u32>::with_capacity(width as usize);
            row.resize(width as usize, 0);
            sk.push(row);
        }

        Sketch { width, depth, sk }
    }

    pub fn add(&mut self, b: &[u8], count: u32) {
        let h1 = leveldb_bloom_hash(b);
        let h2 = jenkins_hash(b);

        for i in 0..self.depth as usize {
            let pos = h1.wrapping_add(h2.wrapping_mul(i as u32)) as usize % (self.width as usize);
            self.sk[i][pos] += count;
        }
    }

    pub fn count(&self, b: &[u8]) -> u32 {
        let h1 = leveldb_bloom_hash(b);
        let h2 = jenkins_hash(b);

        let mut val = u32::MAX;

        for i in 0..self.depth as usize {
            let pos = h1.wrapping_add(h2.wrapping_mul(i as u32)) as usize % (self.width as usize);
            val = val.min(self.sk[i][pos]);
        }

        val
    }

    pub fn merge(&mut self, other: &Sketch) -> Result<(), MergeError> {
        if self.width != other.width || self.depth != other.depth {
            return Err(MergeError::DimensionMismatch);
        }

        for (i, row) in self.sk.iter_mut().enumerate() {
            for (j, w) in row.iter_mut().enumerate() {
                *w += other.sk[i][j]
            }
        }

        Ok(())
    }

    pub fn compress(&mut self) {
        self.width /= 2;

        for d in 0..self.depth as usize {
            let mut row = Vec::<u32>::with_capacity(self.width as usize);
            row.resize(self.width as usize, 0);
            for w in 0..self.width as usize {
                row[w] = self.sk[d][w] + self.sk[d][self.width as usize + w];
            }
            self.sk[d] = row;
        }
    }
}
