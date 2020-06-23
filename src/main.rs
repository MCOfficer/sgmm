#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate colour;

mod remote;

use clap::{App, Arg};
use regex::Regex;
use std::fs;
use std::io::{BufReader, Cursor, Read, Write};
use std::path::PathBuf;

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
                .about("The mod id (1234567890) or workshop URL (https://steamcommunity.com/sharedfiles/filedetails/?id=1234567890)")
                .multiple(true)
                .required(true),
        )
        .arg(Arg::with_name("verbose").short('v').about("Be verbose"))
        .get_matches();

    for modification in matches.values_of("mod").unwrap() {
        let operation = matches.value_of("operation").unwrap();
        let verbose = matches.is_present("verbose");
        let item_id = parse_item_id(modification);

        if operation == "install" {
            install(item_id, verbose);
        } else {
            remove(item_id, verbose);
        }
    }

    green_ln!("{}", "\nDone!");
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
    cyan_ln!("\nInstalling mod {}\n", item_id);

    let paths = build_paths(item_id, verbose);
    let info = remote::steam::retrieve_info(item_id, verbose);

    cyan_ln!("\n### Downloading ###");
    let download_link = remote::get_download_link(item_id, verbose);

    println!("Downloading {}", download_link);
    let res = ureq::get(&download_link).call();
    let mut bytes: Vec<u8> = vec![];
    BufReader::new(res.into_reader())
        .read_to_end(&mut bytes)
        .unwrap();

    cyan_ln!("\n### Installing ###");

    zip_extract::extract(Cursor::new(bytes), &paths.target_dir, true).unwrap();

    println!("Writing .mod file");
    fs::File::create(paths.mod_file)
        .unwrap()
        .write_all(
            format!(
                "name=\"{}\"\npath=\"{}\"",
                info.title,
                paths.target_dir.to_string_lossy()
            )
            .as_bytes(),
        )
        .unwrap();

    println!("Removing mods_registry.json");
    yellow_ln!(
        "Start the Launcher to regenerate it; until then, Stellaris won't recognize your mods."
    );
    if paths.mods_registry.exists() {
        fs::remove_file(paths.mods_registry).unwrap();
    }
}

fn remove(item_id: u32, verbose: bool) {
    println!("\nRemoving mod {}", item_id);

    let paths = build_paths(item_id, verbose);

    if let Err(e) = fs::remove_dir_all(&paths.target_dir) {
        red_ln!(
            "Failed to remove {}; {}",
            paths.target_dir.to_string_lossy(),
            e
        )
    };
    if let Err(e) = fs::remove_file(&paths.mod_file) {
        red_ln!(
            "Failed to remove {}; {}",
            paths.mod_file.to_string_lossy(),
            e
        )
    };
}

struct InstallPaths {
    target_dir: PathBuf,
    mod_file: PathBuf,
    mods_registry: PathBuf,
}

fn build_paths(item_id: u32, verbose: bool) -> InstallPaths {
    if verbose {
        println!("Building paths");
    }
    let stellaris_mods_dir = PathBuf::from(format!(
        "{}/.local/share/Paradox Interactive/Stellaris/mod/",
        std::env::var("HOME").unwrap()
    ));

    let mut target_dir = stellaris_mods_dir.clone();
    target_dir.push(format!("steam_{}", item_id));
    let mod_file = target_dir.with_extension("mod");

    let mut mods_registry = stellaris_mods_dir.parent().unwrap().to_path_buf();
    mods_registry.push("mods_registry.json");

    InstallPaths {
        target_dir,
        mod_file,
        mods_registry,
    }
}
