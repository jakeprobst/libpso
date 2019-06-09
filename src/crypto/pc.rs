// implementation taken from kohle's newserv
// https://github.com/fuzziqersoftware/newserv/

use crate::crypto::{PSOCipher, CipherError};
use std::num::Wrapping as W;

const PC_STREAM_LENGTH: usize = 57;

struct PSOPCCipher {
  stream: [u32; PC_STREAM_LENGTH],
  offset: u16,
}

impl PSOPCCipher {
    pub fn new(seed: u32) -> PSOPCCipher {
        let mut esi: W<u32>;
        let mut ebx: W<u32>;
        let mut edi: W<u32>;
        let mut eax: W<u32>;
        let mut edx: W<u32>;
        let mut var1: W<u32>;

        let mut stream: [u32; PC_STREAM_LENGTH] = [0; PC_STREAM_LENGTH];
        
        esi = W(1);
        ebx = W(seed);
        edi = W(0x15);
        stream[56] = ebx.0;
        stream[55] = ebx.0;
        while edi <= W(0x46E) {
            eax = edi;
            var1 = eax / W(55);
            edx = eax - (var1 * W(55));
            ebx = ebx - esi;
            edi = edi + W(0x15);
            stream[edx.0 as usize] = esi.0;
            esi = ebx;
            ebx = W(stream[edx.0 as usize]);
        }

        let mut cipher = PSOPCCipher {
            stream: stream,
            offset: 1,
        };
        
        for _ in 0..5 {
            cipher.update_stream();
        }

        cipher
    }
   
    fn update_stream(&mut self) {
        let mut esi: W<u32>;
        let mut edi: W<u32>;
        let mut eax: W<u32>;
        let mut ebp: W<u32>;
        let mut edx: W<u32>;

        edi = W(1);
        edx = W(0x18);
        eax = edi;
        while edx > W(0) {
            esi = W(self.stream[eax.0 as usize + 0x1F]);
            ebp = W(self.stream[eax.0 as usize]);
            ebp = ebp - esi;
            self.stream[eax.0 as usize] = ebp.0;
            eax += W(1);
            edx -= W(1);
        }
        edi = W(0x19);
        edx = W(0x1F);
        eax = edi;
        while edx > W(0) {
            esi = W(self.stream[eax.0 as usize - 0x18]);
            ebp = W(self.stream[eax.0 as usize]);
            ebp = ebp - esi;
            self.stream[eax.0 as usize] = ebp.0;
            eax += W(1);
            edx -= W(1);
        }
    }

    fn next(&mut self) -> u32 {
        if self.offset as usize == PC_STREAM_LENGTH {
            self.update_stream();
            self.offset = 1;
        }

        let result = self.stream[self.offset as usize];
        self.offset += 1;
        result
    }
}

impl PSOCipher for PSOPCCipher {
    fn encrypt(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, CipherError> {
        let mut result = Vec::new();
        if data.len() % 4 != 0 {
            return Err(CipherError::InvalidSize)
        }

        for c in data.chunks(4) {
            let mut data = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
            data ^= self.next();
            result.extend_from_slice(&u32::to_le_bytes(data));
        }
        Ok(result)
    }
    
    fn decrypt(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, CipherError> {
        self.encrypt(data)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_crypto() {
        use rand::{Rng, RngCore};
        use super::{PSOCipher, PSOPCCipher};

        let mut rng = rand::thread_rng();

        let seed: u32 = rng.gen();
        let mut cipher_in = PSOPCCipher::new(seed);
        let mut cipher_out = PSOPCCipher::new(seed);

        for _ in 0..10 {
            let mut random_junk = vec![0u8; 40];
            rng.fill_bytes(&mut random_junk);
        
            let enc_data = cipher_in.encrypt(&random_junk).unwrap();
            let orig_data = cipher_out.encrypt(&enc_data).unwrap();
            
            assert!(random_junk == orig_data);
        }
    }
}








