use crate::crypto::{PSOCipher, CipherError};
use std::num::Wrapping as W;

const BB_SBOX_COUNT: usize = 4;
const BB_SBOX_SIZE: usize = 256;
const BB_PARRAY_SIZE: usize = 18;
const BB_SEED_SIZE: usize = 48;

pub struct PSOBBCipher {
    p_array: [u32; BB_PARRAY_SIZE],
    sbox: [[u32; BB_SBOX_SIZE]; BB_SBOX_COUNT],
}

impl PSOBBCipher {
    fn f(&self, x: u32) -> u32 {
        let mut k = W(self.sbox[0][((x >> 24) & 0xFF) as usize]);
        k += W(self.sbox[1][((x >> 16) & 0xFF) as usize]);
        k ^= W(self.sbox[2][((x >> 8) & 0xFF) as usize]);
        k += W(self.sbox[3][((x) & 0xFF) as usize]);
        k.0
    }

    fn init_block(&self, mut l: u32, mut r: u32) -> (u32, u32) {
        for i in (0..16).step_by(2) {
            l ^= self.p_array[i];
            r ^= self.f(l);
            r ^= self.p_array[i+1];
            l ^= self.f(r);
        }
        l ^= self.p_array[16];
        r ^= self.p_array[17];

        (r, l)
    }

    pub fn new(p: [u32; BB_PARRAY_SIZE], s: [[u32; BB_SBOX_SIZE]; BB_SBOX_COUNT], mut seed: [u8; BB_SEED_SIZE]) -> PSOBBCipher {
        for chunk in seed.chunks_mut(3) {
            chunk[0] ^= 0x19;
            chunk[1] ^= 0x16;
            chunk[2] ^= 0x18;
        }

        let mut cipher = PSOBBCipher {
            p_array: p,
            sbox: s
        };

        for k in cipher.p_array.iter_mut() {
            let mut pt = *k as u16;
            pt = ((pt & 0x00FF) << 8) + ((pt & 0xFF00) >> 8);
            *k = ((((*k >> 16) ^ pt as u32) << 16)) + pt as u32;
        }

        for i in 0..18 {
            let k = u32::from_le_bytes([seed[(i * 4 + 3) % BB_SEED_SIZE],
                                        seed[(i * 4 + 2) % BB_SEED_SIZE],
                                        seed[(i * 4 + 1) % BB_SEED_SIZE],
                                        seed[(i * 4) % BB_SEED_SIZE]]);
            cipher.p_array[i] ^= k;
        }

        let mut l = 0;
        let mut r = 0;
        for k in (0..BB_PARRAY_SIZE).step_by(2) {
            // pls rust let me destructure without messing up scope
            let tmp = cipher.init_block(l, r);
            l = tmp.0;
            r = tmp.1;
            cipher.p_array[k] = l;
            cipher.p_array[k + 1] = r;
        }

        for block in 0..BB_SBOX_COUNT {
            for item in (0..BB_SBOX_SIZE).step_by(2) {
                let tmp = cipher.init_block(l, r);
                l = tmp.0;
                r = tmp.1;
                cipher.sbox[block][item] = l;
                cipher.sbox[block][item + 1] = r;
            }
        }

        cipher
    }
}

impl PSOCipher for PSOBBCipher {
    fn encrypt(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, CipherError> {
        let mut real_data = data.chunks(4).map(|k| {
            u32::from_le_bytes([k[0], k[1], k[2], k[3]])
        }).collect::<Vec<_>>();
        if real_data.len() % 2 == 1 {
            real_data.push(0);
        }

        let mut result = Vec::new();
        for d in real_data.chunks(2) {
            let mut l = d[0];
            let mut r = d[1];
            for i in (0..4).step_by(2) {
                l ^= self.p_array[i];
                r ^= self.f(l);
                r ^= self.p_array[i + 1];
                l ^= self.f(r);
            }
            l ^= self.p_array[4];
            r ^= self.p_array[5];

            let tmp = l;
            l = r;
            r = tmp;

            result.extend_from_slice(&l.to_le_bytes());
            result.extend_from_slice(&r.to_le_bytes());
        }

        Ok(result)
    }

    fn decrypt(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, CipherError> {
        if data.len() % 8 != 0 {
            return Err(CipherError::InvalidSize);
        }

        let real_data = data.chunks(4).map(|k| {
            u32::from_le_bytes([k[0], k[1], k[2], k[3]])
        }).collect::<Vec<_>>();

        let mut result = Vec::new();
        for d in real_data.chunks(2) {
            let mut l = d[0];
            let mut r = d[1];
            for i in (1..=4).rev().step_by(2) {
                l ^= self.p_array[i + 1];
                r ^= self.f(l);
                r ^= self.p_array[i];
                l ^= self.f(r);
            }
            l ^= self.p_array[1];
            r ^= self.p_array[0];

            let tmp = l;
            l = r;
            r = tmp;

            result.extend_from_slice(&l.to_le_bytes());
            result.extend_from_slice(&r.to_le_bytes());
        }

        Ok(result)
    }

    fn header_size(&self) -> usize {
        8
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_crypto() {
        use rand::{Rng, RngCore};
        use super::{PSOCipher, PSOBBCipher};

        let mut rng = rand::thread_rng();

        let mut p = [0u32; 18];
        let mut s = [[0u32; super::BB_SBOX_SIZE]; super::BB_SBOX_COUNT];
        let mut seed = [0u8; super::BB_SEED_SIZE];

        rng.fill(&mut p[..]);
        rng.fill(&mut s[0][..]);
        rng.fill(&mut s[1][..]);
        rng.fill(&mut s[2][..]);
        rng.fill(&mut s[3][..]);
        rng.fill(&mut seed[..]);

        let mut cipher_in = PSOBBCipher::new(p, s, seed);
        let mut cipher_out = PSOBBCipher::new(p, s, seed);

        for _ in 0..50 {
            let len = (rng.gen::<u16>() / 4) * 4;

            let mut random_junk = vec![0u8; len as usize];
            rng.fill_bytes(&mut random_junk);

            let enc_data = cipher_in.encrypt(&random_junk).unwrap();
            let orig_data = cipher_out.decrypt(&enc_data).unwrap();
            assert!(random_junk == orig_data[..len as usize].to_vec());
        }
    }
}
