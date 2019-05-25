use std::io::prelude::*;

use failure::Error;
use serde::{Deserialize, Serialize};

use serde_bencode::de;
use serde_bytes::ByteBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Torrent {
    pub announce: String,
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]

    fn parse_string_torrent() {
        let file = File::open("tests/test.torrent").unwrap();

        let torrent = parse(&mut BufReader::new(file)).unwrap();
        assert_eq!(torrent.announce, "http://nyaa.tracker.wf:7777/announce");
        assert!(torrent.announce_list.is_some());
        assert_eq!(torrent.creation_date, 1276147560);

        assert_eq!(torrent.info.name, "[CrunchyRip]_heroman_heroman_pv.ass");
        assert_eq!(
            torrent.info.pieces,
            [
                179, 44, 185, 20, 5, 96, 4, 178, 51, 254, 139, 204, 87, 213, 125, 68, 213, 108, 85,
                199
            ],
        );
        assert_eq!(torrent.info.piece_length, 262144);
        assert_eq!(torrent.info.length.unwrap(), 2084);
    }
}
