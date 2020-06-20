#[macro_use]
extern crate anyhow;

mod remote;

use clap::{App, Arg};
use colored::Colorize;
use regex::Regex;
use std::io::{BufReader, Cursor, Read, Write};
use std::path::PathBuf;
use std::{fs, io};
use zip;

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
                .required(true),
        )
        .arg(Arg::with_name("verbose").short('v').about("Be verbose"))
        .get_matches();

    let operation = matches.value_of("operation").unwrap();
    let modification = matches.value_of("mod").unwrap();
    let verbose = matches.is_present("verbose");
    let item_id = parse_item_id(modification);

    if operation == "install" {
        install(item_id, verbose);
    } else {
        remove(item_id, verbose);
    }

    println!("{}", "\nDone!".green());
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
    println!("Installing mod {}\n", item_id);

    let paths = build_paths(item_id, verbose);
    let info = remote::steam::retrieve_info(item_id, verbose);

    println!("{}", "\n### Downloading ###".cyan());
    let download_link = remote::get_download_link(item_id, verbose);

    println!("Downloading {}", download_link);
    let res = ureq::get(&download_link).call();
    let mut bytes: Vec<u8> = vec![];
    BufReader::new(res.into_reader())
        .read_to_end(&mut bytes)
        .unwrap();

    println!("{}", "\n### Installing ###".cyan());

    extract(bytes, &paths.target_dir, verbose);

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

    println!("Removing mods_registry.json\nOpen the Launcher once to regenerate it; until then, Stellaris won't recognize your mods.");
    if paths.mods_registry.exists() {
        fs::remove_file(paths.mods_registry).unwrap();
    }
}

fn remove(item_id: u32, verbose: bool) {
    println!("Removing mod {}", item_id);

    let paths = build_paths(item_id, verbose);

    if let Err(e) = fs::remove_dir_all(&paths.target_dir) {
        println!(
            "Failed to remove {}; {}",
            paths.target_dir.to_string_lossy(),
            e
        )
    };
    if let Err(e) = fs::remove_file(&paths.mod_file) {
        println!(
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

fn extract(bytes: Vec<u8>, target_dir: &PathBuf, verbose: bool) {
    println!("Extracting to {}", target_dir.to_string_lossy());
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).unwrap();
    }

    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();

    let has_toplevel = has_toplevel(&mut archive);
    if verbose {
        println!("Archive has toplevel dir, stripping")
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let mut relative_path = file.sanitized_name();
        if has_toplevel {
            let base = relative_path
                .components()
                .take(1)
                .fold(PathBuf::new(), |mut p, c| {
                    p.push(c);
                    p
                });
            relative_path = relative_path.strip_prefix(base).unwrap().to_path_buf()
        }

        if relative_path.to_string_lossy().is_empty() {
            // Top-level directory
            continue;
        }

        let mut outpath = target_dir.clone();
        outpath.push(relative_path);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }
    if verbose {
        println!("Extracted {} files", archive.len());
    }
}

fn has_toplevel(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>) -> bool {
    let mut toplevel_dir: Option<PathBuf> = None;
    if archive.len() < 2 {
        return false;
    }

    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap().sanitized_name();
        if let Some(toplevel_dir) = &toplevel_dir {
            if !file.starts_with(toplevel_dir) {
                return false;
            }
        } else {
            // First iteration
            toplevel_dir = Some(file.components().take(1).collect());
        }
    }
    true
}
