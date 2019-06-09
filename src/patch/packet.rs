use psopacket::pso_packet;
use crate::{PSOPacket, PacketParseError};

use std::io::{Read, Seek, SeekFrom};

#[allow(non_camel_case_types)]
type u8_str = u8;

#[pso_packet(0x02)]
pub struct PatchWelcome {
    copyright: [u8_str; 44],
    padding: [u8; 20],
    server_key: u32,
    client_key: u32,
}

impl PatchWelcome {
    fn new(server_key: u32, client_key: u32) -> PatchWelcome {
        PatchWelcome {
            copyright: b"Patch Server. Copyright SonicTeam, LTD. 2001".clone(),
            padding: [0; 20],
            server_key: server_key,
            client_key: client_key,
        }
    }
}

pub enum PatchPacket {
    PatchWelcome(PatchWelcome),
}
 

#[cfg(test)]
mod tests {
    #[test]
    fn patch_welcome() {
        use super::PSOPacket;

        let pkt = super::PatchWelcome::new(123, 456);

        assert!(pkt.as_bytes() == vec![0x4C, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x61, 0x74, 0x63, 0x68, 0x20, 0x53, 0x65,
                                       0x72, 0x76, 0x65, 0x72, 0x2E, 0x20, 0x43, 0x6F, 0x70, 0x79, 0x72, 0x69, 0x67, 0x68, 0x74, 0x20,
                                       0x53, 0x6F, 0x6E, 0x69, 0x63, 0x54, 0x65, 0x61, 0x6D, 0x2C, 0x20, 0x4C, 0x54, 0x44, 0x2E, 0x20,
                                       0x32, 0x30, 0x30, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                                       0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7B, 0x00, 0x00, 0x00, 0xC8, 0x01, 0x00, 0x00,
        ]);


        let mut bytes = pkt.as_bytes();
        bytes.splice(32..41, b"Elsewhere".iter().cloned());

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
}
