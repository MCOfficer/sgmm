#[macro_use]
extern crate anyhow;

mod remote;

use clap::{App, Arg};
use pbr;
use progress_streams::ProgressReader;
use regex::Regex;
use std::io::{BufReader, Read};

fn main() {
    let matches = App::new("Stellaris GOG Mod Manager (sgmm)")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("operation")
                .about("The operation")
                .required(true)
                .possible_values(&vec!["install", "remove"]),
        )
        .arg(
            Arg::with_name("mod")
                .about("The mod id (1234567890)")
                .required(true),
        )
        .arg(Arg::with_name("verbose").short('v').about("Be verbose"))
        .get_matches();

    let operation = matches.value_of("operation").unwrap();
    let modification = matches.value_of("mod").unwrap();
    let verbose = matches.is_present("verbose");
    if operation == "install" {
        install(parse_item_id(modification), verbose);
    } else {
        remove(modification, verbose);
    }
}

fn parse_item_id(str: &str) -> u32 {
    match str.parse::<u32>() {
        Ok(id) => id,
        Err(_) => {
            let re = Regex::new(r"https://steamcommunity.com/sharedfiles/filedetails/\?id=(\d+)")
                .unwrap();
            match re.captures(str) {
                None => panic!("Failed to parse {}", str),
                Some(captures) => captures.get(1).unwrap().as_str().parse::<u32>().unwrap(),
            }
        }
    }
}

fn install(item_id: u32, verbose: bool) {
    println!("Installing mod {}", item_id);

    println!("Fetching info");
    let file_info = match remote::steam::retrieve_info(item_id, verbose) {
        Ok(i) => i,
        Err(e) => panic!("Failed to get info from steam: {}", e),
    };

    let download_link = remote::get_download_link(item_id, verbose);

    println!("Downloading {}", download_link);
    let res = ureq::get(&download_link).call();
    let mut bytes: Vec<u8> = Vec::with_capacity(file_info.file_size);
    BufReader::new(res.into_reader())
        .read_to_end(&mut bytes)
        .unwrap();

    // Unpack to ~/.local/share/Paradox Interactive/Stellaris/mod/steam_{id}
    // Create ~/.local/share/Paradox Interactive/Stellaris/mod/steam_{id}.mod with:
    //      name="{name}"
    //      path="{full path}"
    // remove ~/.local/share/Paradox Interactive/Stellaris/mods_registry.json
}

fn remove(modification: &str, verbose: bool) {
    println!("Removing mod {}", modification);
    unimplemented!()
}
