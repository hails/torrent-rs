#![warn(unused_extern_crates)]

use failure::Error;
use std::env;
use std::fs::File;
use std::io::BufReader;

mod announce;
mod torrent_info;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    let file = File::open(&args[1])?;

    let torrent = torrent_info::parse(&mut BufReader::new(file))?;

    let res = announce::announce(announce::generate_announce(&torrent)?, &torrent.announce)?;

    println!(
        "Announced torrent << {} >> on `{}`",
        torrent.info.name, torrent.announce
    );

    println!("Response: {:#?}", res);

    Ok(())
}
