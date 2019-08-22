pub mod pc;
pub mod bb;


#[derive(Debug)]
pub enum CipherError {
    InvalidSize
}



pub trait PSOCipher {
    fn encrypt(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, CipherError>;
    fn decrypt(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, CipherError>;
    fn header_size(&self) -> usize;
    fn block_size(&self) -> usize {
        self.header_size()
    }
}



pub struct NullCipher {
}

impl PSOCipher for NullCipher {
    fn encrypt(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, CipherError> {
        Ok(data.clone())
    }

    fn decrypt(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, CipherError> {
        Ok(data.clone())
    }

    fn header_size(&self) -> usize {
        4
    }
}
