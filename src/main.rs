#![warn(unused_extern_crates)]

use failure::Error;
use std::env;
use std::fs::File;
use std::io::BufReader;

mod torrent;
mod tracker;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    let file = File::open(&args[1])?;

    let torrent = torrent::parse(&mut BufReader::new(file))?;

    let res = tracker::announce(tracker::generate_announce(&torrent)?, &torrent.announce)?;

    println!(
        "Announced torrent << {} >> on `{}`",
        torrent.info.name, torrent.announce
    );

    println!("Response: {:#?}", res);

    Ok(())
}
