mod patch;

#[derive(Debug, PartialEq)]
pub enum PacketParseError {
    NotEnoughBytes,
    WrongPacketCommand,
}


pub trait PSOPacket {
    fn from_bytes(data: &Vec<u8>) -> Result<Self, PacketParseError> where Self: Sized;
    fn as_bytes(&self) -> Vec<u8>;
}



