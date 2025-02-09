use cbc::cipher::BlockEncryptMut;
use des::cipher::block_padding::ZeroPadding;
use des::cipher::BlockSizeUser;
use des::cipher::KeyIvInit;
use rand::RngCore;
use tiger::{Digest, Tiger};

type TdesCbcEnc = cbc::Encryptor<des::TdesEde3>;
// type TdesCbcDec = cbc::Decryptor<des::TdesEde3>;

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

pub fn encrypt_buffer_in_place(buf: &mut Vec<u8>, key: &[u8; 24], iv: &[u8; 8]) {
    let buf_len = buf.len();
    buf.resize(buf_len.next_multiple_of(des::TdesEde3::block_size()), 0);

    let encrypted = TdesCbcEnc::new(key.into(), iv.into())
        .encrypt_padded_mut::<ZeroPadding>(buf.as_mut_slice(), buf_len)
        .unwrap();

    debug_assert_eq!(encrypted.len(), buf.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_calculates_iv() {
        const SEED: u32 = 3223919485;
        const EXPECTED_IV: [u8; 8] = [242, 25, 90, 22, 129, 137, 67, 189];

        let iv = generate_iv_from_seed(SEED);

        assert_eq!(iv, EXPECTED_IV);
    }

    #[test]
    fn correctly_encrypts_buffer() {
        const KEY: [u8; 24] = [
            92, 21, 207, 202, 121, 14, 132, 211, 96, 205, 189, 107, 35, 136, 108, 251, 158, 122,
            218, 52, 169, 195, 1, 222,
        ];
        const SEED: u32 = 12345678u32;
        const EXPECTED_OUTPUT: [u8; 48] = [
            78, 175, 165, 216, 49, 54, 245, 194, 136, 92, 151, 42, 82, 14, 111, 239, 84, 101, 39,
            248, 187, 165, 190, 145, 88, 28, 127, 158, 76, 227, 32, 11, 65, 36, 53, 240, 192, 26,
            231, 40, 43, 33, 246, 155, 3, 135, 185, 123,
        ];
        let mut buf: Vec<u8> = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41,
        ];
        let iv = generate_iv_from_seed(SEED);
        encrypt_buffer_in_place(&mut buf, &KEY, &iv);

        assert_eq!(buf.as_slice(), EXPECTED_OUTPUT);
    }
}
