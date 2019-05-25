use failure::Error;
use serde::{Deserialize, Serialize};

use serde_bencode::de;
use serde_bytes::ByteBuf;
use serde_urlencoded;

use sha1::Sha1;

use rand::Rng;

use crate::torrent::Torrent;

use reqwest;

use percent_encoding::percent_encode_byte;

use itertools::Itertools;

#[derive(Serialize, Debug, PartialEq)]
pub struct Announce {
    #[serde(skip_serializing)]
    pub info_hash: String,
    #[serde(skip_serializing)]
    info_hash_bytes: [u8; 20],
    pub peer_id: String,
    pub uploaded: u32,
    pub downloaded: u32,
    pub left: u32,
    pub port: u32,
    pub compact: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    #[serde(rename = "failure reason")]
    failure_reason: Option<String>,
    pub complete: Option<u32>,
    pub incomplete: Option<u32>,
    pub interval: Option<u32>,
    #[serde(rename = "peers")]
    peers_bin: Option<ByteBuf>,
    pub peers: Option<Vec<Peer>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Peer {
    pub ip: String,
    pub port: u16,
}

pub fn announce(announce_info: Announce, tracker_url: &str) -> Result<Response, Error> {
    let announce_info = Announce {
        info_hash: announce_info
            .info_hash_bytes
            .iter()
            .map(|byte| percent_encode_byte(*byte))
            .collect(),
        ..announce_info
    };

    let qs = serde_urlencoded::to_string(&announce_info).unwrap();

    let announce_url = format!(
        "{}?{}&info_hash={}",
        tracker_url, qs, announce_info.info_hash
    );

    let mut response = reqwest::get(&announce_url)?;
    let mut buf: Vec<u8> = vec![];
    response.copy_to(&mut buf)?;

    let mut tracker_response: Response = de::from_bytes(&buf)?;

    match &tracker_response.failure_reason {
        Some(failure) => panic!("{:?}", failure),
        None => (),
    }

    tracker_response.peers = match &tracker_response.peers_bin {
        Some(peers) => Some(parse_peers(&peers)),
        None => None,
    };

    Ok(tracker_response)
}

fn parse_peers(peers: &ByteBuf) -> Vec<Peer> {
    let mut parsed_peers = vec![];

    for mut chunk in &peers.into_iter().chunks(6) {
        let ip: String = format!("{}", chunk.by_ref().take(4).format("."));
        let port: Vec<_> = chunk.take(2).collect();

        parsed_peers.push(Peer {
            ip,
            port: u16::from(*port[0]) << 8 | u16::from(*port[1]),
        })
    }

    parsed_peers
}

pub fn generate_announce(torrent: &Torrent) -> Result<Announce, Error> {
    let torrent_info = serde_bencode::to_bytes(&torrent.info)?;
    let info_hash = Sha1::from(&torrent_info).digest();

    let peer_id = format!("-RS0001-{}", random_numbers());
    assert!(peer_id.len() == 20, "peer_id should have 20 bytes");

    Ok(Announce {
        info_hash: "".to_owned(),
        info_hash_bytes: info_hash.bytes(),
        peer_id,
        uploaded: 0,
        downloaded: 0,
        left: 0,
        port: 43254,
        compact: "1".to_string(),
    })
}

fn random_numbers() -> String {
    const CHARSET: &[u8] = b"0123456789";
    const PASSWORD_LEN: usize = 12;
    let mut rng = rand::thread_rng();

    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            // This is safe because `idx` is in range of `CHARSET`
            char::from(unsafe { *CHARSET.get_unchecked(idx) })
        })
        .collect();

    password
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_bytes::ByteBuf;

    #[test]
    fn generate_announce_correctly() {
        use crate::torrent::Info;

        let peer_id_start = "-RS0001-";

        let torrent = Torrent {
            announce: "http://nyaa.tracker.wf:7777/announce".to_string(),
            announce_list: None,
            creation_date: 1276147560,
            info: Info {
                name: "[CrunchyRip]_heroman_heroman_pv.ass".to_string(),
                pieces: ByteBuf::from(vec![
                    179, 44, 185, 20, 5, 96, 4, 178, 51, 254, 139, 204, 87, 213, 125, 68, 213, 108,
                    85, 199,
                ]),
                piece_length: 262144,
                length: Some(2084),
                md5sum: None,
                files: None,
            },
        };

        let announce = generate_announce(&torrent).unwrap();

        assert_eq!(announce.info_hash, "");
        assert_eq!(
            announce.info_hash_bytes,
            [
                184, 215, 59, 115, 255, 110, 150, 183, 133, 116, 91, 126, 68, 166, 246, 33, 123,
                87, 207, 219
            ]
        );
        assert!(
            announce.peer_id.starts_with(peer_id_start),
            "peer_id should start with value {}",
            peer_id_start
        );
        assert_eq!(announce.uploaded, 0);
        assert_eq!(announce.downloaded, 0);
        assert_eq!(announce.left, 0);
        assert_eq!(announce.port, 43254);
        assert_eq!(announce.compact, "1");
    }

    #[test]
    fn parse_peers_correctly() {
        let peers_bin = [127, 0, 0, 1, 185, 141, 192, 168, 0, 1, 168, 246];
        let peers = parse_peers(&ByteBuf::from(peers_bin.to_vec()));

        assert_eq!(peers[0].ip, String::from("127.0.0.1"));
        assert_eq!(peers[0].port, 47501 as u16);

        assert_eq!(peers[1].ip, String::from("192.168.0.1"));
        assert_eq!(peers[1].port, 43254 as u16);
    }

    #[test]

    fn parse_peers_empty() {
        let peers_bin = [];
        let peers = parse_peers(&ByteBuf::from(peers_bin.to_vec()));

        assert!(peers.is_empty());
    }
}
