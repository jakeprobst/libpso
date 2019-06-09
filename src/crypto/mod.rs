mod pc;


#[derive(Debug)]
pub enum CipherError {
    InvalidSize
}



trait PSOCipher {
    fn encrypt(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, CipherError>;
    fn decrypt(&mut self, data: &Vec<u8>) -> Result<Vec<u8>, CipherError>;
}
