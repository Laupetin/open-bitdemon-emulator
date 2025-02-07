use rand::RngCore;
use tiger::{Digest, Tiger};

pub fn generate_iv_seed() -> u32 {
    rand::rng().next_u32()
}

pub fn generate_iv_from_seed(seed: u32) -> [u8; 8] {
    let mut tiger = Tiger::new();
    tiger.update(seed.to_le_bytes());
    let a: [u8; 24] = tiger.finalize().into();
    let mut b: [u8; 8] = [0; 8];
    b.copy_from_slice(&a[0..8]);

    b
}
