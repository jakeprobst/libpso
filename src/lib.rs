pub mod crypto;
pub mod packet;
pub mod character;

#[derive(Debug, PartialEq)]
pub enum PacketParseError {
    NotEnoughBytes,
    WrongPacketCommand,
    WrongPacketForServerType,
    WrongPacketSize(u16, usize),
    DataStructNotLargeEnough(u64, usize),
    InvalidValue,
}


pub trait PSOPacket: std::fmt::Debug {
    fn from_bytes(data: &Vec<u8>) -> Result<Self, PacketParseError> where Self: Sized;
    fn as_bytes(&self) -> Vec<u8>;
}



