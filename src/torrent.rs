use std::io::prelude::*;

use failure::Error;
use serde::{Deserialize, Serialize};

use serde_bencode::de;
use serde_bytes::ByteBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Torrent {
    pub announce: String,
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<String>>,
    #[serde(rename = "creation date")]
    pub creation_date: i32,
    pub info: Info,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct Info {
    pub name: String,
    pub pieces: ByteBuf,
    #[serde(rename = "piece length")]
    pub piece_length: u32,
    pub length: Option<u32>,
    pub md5sum: Option<String>,
    pub files: Option<Vec<File>>,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct File {
    pub length: u32,
    pub md5sum: Option<String>,
    pub path: String,
}

pub fn parse(torrent: &mut BufRead) -> Result<Torrent, Error> {
    let mut contents = Vec::new();
    torrent.read_to_end(&mut contents)?;

    let parsed = de::from_bytes::<Torrent>(&contents)?;

    Ok(parsed)
}
