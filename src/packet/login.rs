use psopacket::pso_packet;
use crate::{PSOPacket, PacketParseError};

use std::io::{Read, Seek, SeekFrom};

pub const PATCH_FILE_CHUNK_SIZE: u16 = 0x8000; // 32kb

#[allow(non_camel_case_types)]
type u8_str = u8;

#[pso_packet(0x03)]
pub struct LoginWelcome {
    flag: u32,
    copyright: [u8_str; 0x60],
    server_key: [u8; 48],
    client_key: [u8; 48],
}

impl LoginWelcome {
    pub fn new(server_key: [u8; 48], client_key: [u8; 48]) -> LoginWelcome {
        let mut copyright = [0u8; 0x60];
        copyright[..0x4B].clone_from_slice(b"Phantasy Star Online Blue Burst Game Server. Copyright 1999-2004 SONICTEAM.");
        LoginWelcome {
            flag: 0,
            copyright: copyright,
            server_key: server_key,
            client_key: client_key,
        }
    }
}



#[pso_packet(0x93)]
pub struct Login {
    pub flag: u32,
    pub tag: u32,
    pub guildcard: u32,
    pub version: u16,
    unknown1: [u8; 6],
    pub team: u32,
    pub username: [u8_str; 16],
    unknown2: [u8; 32],
    pub password: [u8_str; 16],
    unknown3: [u8; 40],
    pub hwinfo: [u8; 8],
    pub security_data: [u8; 40],
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccountStatus {
    Ok,
    Error,
    InvalidPassword,
    InvalidPassword2,
    Maintenance,
    AlreadyOnline,
    Banned,
    Banned2,
    InvalidUser,
    PayUp,
    Locked,
    BadVersion,
}

impl AccountStatus {
    const SIZE: usize = 4;
    
    fn to_le_bytes(&self) -> [u8; 4] {
        [match self {
            AccountStatus::Ok => 0,
            AccountStatus::Error => 1,
            AccountStatus::InvalidPassword => 2,
            AccountStatus::InvalidPassword2 => 3,
            AccountStatus::Maintenance => 4,
            AccountStatus::AlreadyOnline => 5,
            AccountStatus::Banned => 6,
            AccountStatus::Banned2 => 7,
            AccountStatus::InvalidUser => 8,
            AccountStatus::PayUp => 9,
            AccountStatus::Locked => 10,
            AccountStatus::BadVersion => 11,
        },0,0,0]
    }

    fn from_le_bytes(bytes: [u8; 4]) -> Result<AccountStatus, PacketParseError> {
        match bytes[0] {
            0 => Ok(AccountStatus::Ok),
            1 => Ok(AccountStatus::Error),
            2 => Ok(AccountStatus::InvalidPassword),
            3 => Ok(AccountStatus::InvalidPassword2),
            4 => Ok(AccountStatus::Maintenance),
            5 => Ok(AccountStatus::AlreadyOnline),
            6 => Ok(AccountStatus::Banned),
            7 => Ok(AccountStatus::Banned2),
            8 => Ok(AccountStatus::InvalidUser),
            9 => Ok(AccountStatus::PayUp),
            10 => Ok(AccountStatus::Locked),
            11 => Ok(AccountStatus::BadVersion),
            _ => Err(PacketParseError::InvalidValue),
        }
    }
        
}

#[pso_packet(0xE6)]
pub struct LoginResponse {
    pub flag: u32,
    pub status: AccountStatus,
    pub tag: u32,
    pub guildcard: u32,
    pub team_id: u32,
    pub security_data: [u8; 40],
    pub caps: u32,
}

impl LoginResponse {
    pub fn by_status(status: AccountStatus, security_data: [u8; 40]) -> LoginResponse {
        LoginResponse {
            flag: 0,
            status: status,
            tag: 0x00010000,
            //tag: 0x00000100,
            guildcard: 0,
            team_id: 0,
            security_data: security_data,
            caps: 0x00000102,
        }
    }
}


#[pso_packet(0xE0)]
pub struct RequestSettings {
    flag: u32
}


#[pso_packet(0x19)]
pub struct RedirectClient {
    flag: u32,
    ip: u32,
    port: u16,
    padding: u16,
}

impl RedirectClient {
    pub fn new(ip: u32, port: u16) -> RedirectClient {
        RedirectClient {
            flag: 0,
            ip: ip,
            port: port,
            padding: 0,
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_account_status_enum() {
        use super::PSOPacket;
        let pkt = super::LoginResponse {
            flag: 0,
            status: super::AccountStatus::InvalidPassword,
            tag: 0,
            guildcard: 0,
            team_id: 0,
            security_data: [0; 40],
            caps: 0,
        };

        let mut bytes = pkt.as_bytes();
        assert!(bytes[8] == 2);

        bytes[8] = 8;

        let pkt = super::LoginResponse::from_bytes(&bytes).unwrap();
        assert!(pkt.status == super::AccountStatus::InvalidUser);
    }
}
