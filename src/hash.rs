pub fn leveldb_bloom_hash(b: &[u8]) -> u32 {
    let mut len = b.len() as u32;

    let seed: u32 = 0xbc9f1d34;
    let m: u32 = 0xc6a4a793;

    let mut h = seed ^ len.wrapping_mul(m);

    let mut idx = 0usize;
    while len >= 4 {
        h = h.wrapping_add(
            (b[idx + 0usize] as u32)
                | ((b[idx + 1usize] as u32) << 8)
                | ((b[idx + 2usize] as u32) << 16)
                | ((b[idx + 3usize] as u32) << 24),
        );
        h = h.wrapping_mul(m);
        h ^= h >> 16;
        idx += 4;
        len -= 4;
    }

    match len {
        3 => {
            h += ((b[idx + 2] as u32) << 16) | ((b[idx + 1] as u32) << 8) | (b[idx] as u32);
            h = h.wrapping_mul(m);
            h ^= h >> 24;
        }
        2 => {
            h += ((b[idx + 1] as u32) << 8) | (b[idx] as u32);
            h = h.wrapping_mul(m);
            h ^= h >> 24;
        }
        1 => {
            h += b[idx] as u32;
            h = h.wrapping_mul(m);
            h ^= h >> 24;
        }
        0 => {}
        _ => {}
    }

    return h;
}

pub fn jenkins_hash(b: &[u8]) -> u32 {
    let mut h2 = 0u32;

    for &ch in b {
        h2 = h2.wrapping_add(ch as u32);
        h2 = h2.wrapping_add(h2 << 10);
        h2 ^= h2 >> 6;
    }

    h2 = h2.wrapping_add(h2 << 3);
    h2 ^= h2 >> 11;
    h2 = h2.wrapping_add(h2 << 15);

    return h2;
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    fn smoke_hash(hash: fn(&[u8]) -> u32, golden: &str) {
        let mut b = Vec::new();

        let file = File::open(Path::new(golden)).unwrap();
        let reader = BufReader::new(file);

        let mut i = 0u32;
        for line in reader.lines() {
            if let Ok(want) = line {
                let want32 = u32::from_str_radix(&want, 16).unwrap();
                let h = hash(&b);
                if h != want32 {
                    println!("h={:x} want32={:x} MISMATCH", h, want32);
                }
                b.push(i as u8);
                i += 1;
            }
        }
    }

    #[test]
    fn smoke_leveldb() {
        smoke_hash(leveldb_bloom_hash, "testdata/leveldb.txt");
    }
    #[test]
    fn smoke_jenkins() {
        smoke_hash(jenkins_hash, "testdata/jenkins.txt");
    }
}
