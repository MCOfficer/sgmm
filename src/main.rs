#[macro_use]
extern crate anyhow;

mod remote;

use clap::{App, Arg};
use regex::Regex;

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

    println!("\n### Attempting download via steamworkshop.download ###");
    match remote::steamworkshop_download::request_download(item_id, verbose) {
        Ok(download_link) => {
            println!("Stub: process {}", download_link);
            return;
        }
        Err(e) => println!("Failed to request download: {}", e),
    };

    println!("\n### Attempting download via steamworkshopdownloader.io ###");
    match remote::steamworkshopdownloader_io::request_transfer(item_id, verbose) {
        Ok(download_res) => {
            println!("Stub: process {:#?}", download_res);
            return;
        }
        Err(e) => println!("Failed to request download: {}", e),
    };
}

fn remove(modification: &str, verbose: bool) {
    println!("Removing mod {}", modification);
    unimplemented!()
}
