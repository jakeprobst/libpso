use psopacket::pso_packet;
use crate::{PSOPacket, PacketParseError};

use std::io::{Read, Seek, SeekFrom};

pub const PATCH_FILE_CHUNK_SIZE: u16 = 0x8000; // 32kb

#[allow(non_camel_case_types)]
type u8_str = u8;

// outgoing packets
#[pso_packet(0x02)]
pub struct PatchWelcome {
    copyright: [u8_str; 44],
    padding: [u8; 20],
    server_key: u32,
    client_key: u32,
}

impl PatchWelcome {
    pub fn new(server_key: u32, client_key: u32) -> PatchWelcome {
        PatchWelcome {
            copyright: b"Patch Server. Copyright SonicTeam, LTD. 2001".clone(),
            padding: [0; 20],
            server_key: server_key,
            client_key: client_key,
        }
    }
}


// incoming packets
#[pso_packet(0x02)]
pub struct PatchWelcomeReply {
}

#[pso_packet(0x04)]
pub struct RequestLogin {
}

#[pso_packet(0x04)]
pub struct LoginReply {
    unused: [u8; 12],
    username: [u8_str; 16],
    password: [u8_str; 16],
    unused2: [u8; 64],
}

#[pso_packet(0x06)]
pub struct StartFileSend {
    id: u32,
    size: u32,
    filename: [u8_str; 48],
}

impl StartFileSend {
    pub fn new(filename: &str, size: u32, id: u32) -> StartFileSend {
        let mut f = [0u8; 48];
        for (src, dst) in filename.as_bytes().iter().zip(f.iter_mut()) {
            *dst = *src
        }
        StartFileSend {
            id: id,
            size: size,
            filename: f,
        }
    }
}

//#[pso_packet(0x07)]
pub struct FileSend {
    pub chunk_num: u32,
    pub checksum: u32,
    pub chunk_size: u32,
    pub buffer: [u8; PATCH_FILE_CHUNK_SIZE as usize],
}

impl PSOPacket for FileSend {
    fn from_bytes(_data: &Vec<u8>) -> Result<FileSend, PacketParseError> {
        // TODO: implement this? it shouldn't be called on the server side ever...
        unimplemented!();
    }

    fn as_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&u32::to_le_bytes(self.chunk_num));
        buf.extend_from_slice(&u32::to_le_bytes(self.checksum));
        buf.extend_from_slice(&u32::to_le_bytes(self.chunk_size));
        buf.extend_from_slice(&self.buffer[0..self.chunk_size as usize]);
        while buf.len() % 4 != 0 {
            buf.push(0);
        }
        //buf

        //buf.extend_from_slice(&u16::to_le_bytes((4 * 4) as u16 + self.chunk_size as u16));
        //buf.extend_from_slice(&u16::to_le_bytes(0x07));
        let pkt_len = (buf.len() + 4) as u16;
        let mut prebuf: Vec<u8> = Vec::new();

        prebuf.extend_from_slice(&u16::to_le_bytes(pkt_len));
        prebuf.extend_from_slice(&u16::to_le_bytes(0x07));
        prebuf.append(&mut buf);

        prebuf

    }
}

impl std::fmt::Debug for FileSend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "packet FileSend {{\n").unwrap();
        write!(f, "    chunk_num: {:?}\n", self.chunk_num).unwrap();
        write!(f, "    checksum: {:X?}\n", self.checksum).unwrap();
        write!(f, "    chunk_size: {:X?}\n", self.chunk_size).unwrap();
        write!(f, "    buffer: [...a large array ...]\n").unwrap();
        write!(f, "}}")
    }
}


#[pso_packet(0x08)]
pub struct EndFileSend {
    padding: u32,
}

impl EndFileSend {
    pub fn new() -> EndFileSend {
        EndFileSend {
            padding: 0,
        }
    }
}



#[pso_packet(0x0B)]
pub struct PatchStartList {
}

#[pso_packet(0x09)]
pub struct ChangeDirectory {
    dirname: [u8_str; 64]
}

impl ChangeDirectory {
    pub fn new(dirname: &str) -> ChangeDirectory {
        let mut d = [0u8; 64];
        for (src, dst) in dirname.as_bytes().iter().zip(d.iter_mut()) {
            *dst = *src
        }
        ChangeDirectory {
            dirname: d,
        }
    }
}

#[pso_packet(0x0A)]
pub struct UpOneDirectory {
}

#[pso_packet(0x0C)]
pub struct FileInfo {
    id: u32,
    filename: [u8_str; 32],
}

impl FileInfo {
    pub fn new(filename: &str, id: u32) -> FileInfo {
        let mut f = [0u8; 32];
        for (src, dst) in filename.as_bytes().iter().zip(f.iter_mut()) {
            *dst = *src
        };
        FileInfo {
            id: id,
            filename: f,
        }
    }
}


#[pso_packet(0x0D)]
pub struct PatchEndList {
}

#[pso_packet(0x0F)]
pub struct FileInfoReply {
    pub id: u32,
    pub checksum: u32,
    pub size: u32,
}

#[pso_packet(0x10)]
pub struct FileInfoListEnd {
}

#[pso_packet(0x11)]
pub struct FilesToPatchMetadata {
    data_size: u32,
    file_count: u32,
}

impl FilesToPatchMetadata {
    pub fn new(data_size: u32, file_count: u32) -> FilesToPatchMetadata {
        FilesToPatchMetadata {
            data_size: data_size,
            file_count: file_count,
        }
    }
}


#[pso_packet(0x12)]
pub struct FinalizePatching {
}


#[pso_packet(0x13)]
pub struct Message {
    msg: String,
}

impl Message {
    pub fn new(mut msg: String) -> Message {
        msg.push('\0');
        Message {
            msg: msg,
        }
    }
}


#[pso_packet(0x14)]
pub struct RedirectClient {
    ip: u32,
    port: u16,
    padding: u16,
}

impl RedirectClient {
    pub fn new(ip: u32, port: u16) -> RedirectClient {
        RedirectClient {
            ip: ip,
            port: port,
            padding: 0,
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn patch_welcome() {
        use super::PSOPacket;

        let pkt = super::PatchWelcome::new(123, 456);

        assert!(pkt.as_bytes() == vec![0x4C, 0x00, 0x02, 0x00, 0x50, 0x61, 0x74, 0x63, 0x68, 0x20, 0x53, 0x65,
                                       0x72, 0x76, 0x65, 0x72, 0x2E, 0x20, 0x43, 0x6F, 0x70, 0x79, 0x72, 0x69, 0x67, 0x68, 0x74, 0x20,
                                       0x53, 0x6F, 0x6E, 0x69, 0x63, 0x54, 0x65, 0x61, 0x6D, 0x2C, 0x20, 0x4C, 0x54, 0x44, 0x2E, 0x20,
                                       0x32, 0x30, 0x30, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                       0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7B, 0x00, 0x00, 0x00, 0xC8, 0x01, 0x00, 0x00,
        ]);


        let mut bytes = pkt.as_bytes();
        bytes.splice(28..37, b"Elsewhere".iter().cloned());

        let new_pkt = super::PatchWelcome::from_bytes(&bytes);

        assert!(new_pkt == Ok(super::PatchWelcome {
            copyright: b"Patch Server. Copyright Elsewhere, LTD. 2001".clone(),
            padding: [0; 20],
            server_key: 123,
            client_key: 456,
        }));
        if let Ok(p) = new_pkt {
            println!("{:?}", p);
        }
    }

    #[test]
    fn test_message() {
        use super::PSOPacket;

        let msg = super::Message::new("hello this is an arbitrary message?!!".to_string());

        assert!(msg.as_bytes() == vec![0x50, 0x00, 0x13, 0x00, 0x68, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00, 0x20, 0x00,
                                       0x74, 0x00, 0x68, 0x00, 0x69, 0x00, 0x73, 0x00, 0x20, 0x00, 0x69, 0x00, 0x73, 0x00, 0x20, 0x00,
                                       0x61, 0x00, 0x6E, 0x00, 0x20, 0x00, 0x61, 0x00, 0x72, 0x00, 0x62, 0x00, 0x69, 0x00, 0x74, 0x00,
                                       0x72, 0x00, 0x61, 0x00, 0x72, 0x00, 0x79, 0x00, 0x20, 0x00, 0x6D, 0x00, 0x65, 0x00, 0x73, 0x00,
                                       0x73, 0x00, 0x61, 0x00, 0x67, 0x00, 0x65, 0x00, 0x3F, 0x00, 0x21, 0x00, 0x21, 0x00, 0x00, 0x00]);

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&70u16.to_le_bytes());
        bytes.extend_from_slice(&19u16.to_le_bytes());
        for c in "this is a cool string of letters!".encode_utf16() {
            bytes.extend_from_slice(&c.to_le_bytes());
        }

        let msg = super::Message::from_bytes(&bytes);
        let b = msg.unwrap().as_bytes();
        assert!(b == vec![0x48, 0x00, 0x13, 0x00, 0x74, 0x00, 0x68, 0x00, 0x69, 0x00, 0x73, 0x00, 0x20, 0x00, 0x69, 0x00,
                          0x73, 0x00, 0x20, 0x00, 0x61, 0x00, 0x20, 0x00, 0x63, 0x00, 0x6F, 0x00, 0x6F, 0x00, 0x6C, 0x00,
                          0x20, 0x00, 0x73, 0x00, 0x74, 0x00, 0x72, 0x00, 0x69, 0x00, 0x6E, 0x00, 0x67, 0x00, 0x20, 0x00,
                          0x6F, 0x00, 0x66, 0x00, 0x20, 0x00, 0x6C, 0x00, 0x65, 0x00, 0x74, 0x00, 0x74, 0x00, 0x65, 0x00,
                          0x72, 0x00, 0x73, 0x00, 0x21, 0x00, 0x00, 0x00])
    }
}
