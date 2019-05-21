use failure::Error;
use serde::{Deserialize, Serialize};

use serde_bencode::de;
use serde_bytes::ByteBuf;
use serde_urlencoded;

use sha1::Sha1;

use rand::Rng;

use crate::torrent_info::Torrent;

use reqwest;

use percent_encoding::percent_encode_byte;

use itertools::Itertools;

#[derive(Serialize, Debug)]
pub struct TrackerAnnounce {
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
pub struct TrackerResponse {
    #[serde(rename = "failure reason")]
    failure_reason: Option<String>,
    pub complete: Option<u32>,
    pub incomplete: Option<u32>,
    pub interval: Option<u32>,
    #[serde(rename = "peers")]
    peers_bin: Option<ByteBuf>,
    pub peers: Option<Vec<(String, u16)>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackerResponsePeer {
    pub ip: String,
    // pub port:
}

pub fn announce(
    announce_info: TrackerAnnounce,
    tracker_url: &String,
) -> Result<TrackerResponse, Error> {
    let announce_info = TrackerAnnounce {
        info_hash: announce_info
            .info_hash_bytes
            .into_iter()
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

    let mut tracker_response: TrackerResponse = de::from_bytes(&buf)?;

    match &tracker_response.failure_reason {
        Some(_) => panic!("FAILED"),
        None => (),
    }

    let mut peers: Vec<(String, u16)> = vec![];

    for mut chunk in &tracker_response
        .clone()
        .peers_bin
        .unwrap()
        .into_iter()
        .chunks(6)
    {
        let ip: String = format!("{}", chunk.by_ref().take(4).format("."));
        let port: Vec<_> = chunk.by_ref().take(2).collect();
        peers.push((ip, ((port[0] as u16) << 8 | port[1] as u16)));
    }

    match &tracker_response.peers_bin {
        Some(_) => tracker_response.peers = Some(peers),
        None => tracker_response.peers = None,
    }

    Ok(tracker_response)
}

pub fn generate_announce(torrent: &Torrent) -> Result<TrackerAnnounce, Error> {
    let torrent_info = serde_bencode::to_bytes(&torrent.info)?;
    let info_hash = Sha1::from(&torrent_info).digest();

    let peer_id = format!("-RS0001-{}", random_numbers());
    assert!(peer_id.len() == 20, "peer_id should have 20 bytes");

    Ok(TrackerAnnounce {
        info_hash: "".to_owned(),
        info_hash_bytes: info_hash.bytes(),
        peer_id,
        uploaded: 0,
        downloaded: 0,
        left: 0,
        port: 0,
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
