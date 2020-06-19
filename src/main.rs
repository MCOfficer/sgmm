extern crate clap;
use clap::{App, Arg};

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
                .about("The mod id (1234567890) or link (https://steamcommunity.com/sharedfiles/filedetails/?id=1234567890)")
                .required(true),
        )
        .arg(Arg::with_name("v").short('v').about("Be verbose"))
        .get_matches();

    let operation = matches.value_of("operation").unwrap();
    let modification = matches.value_of("mod").unwrap();
    if operation == "install" {
        install(modification);
    } else {
        remove(modification);
    }
}

fn install(modification: &str) {
    println!("Installing mod {}", modification);
    unimplemented!()
}

fn remove(modification: &str) {
    println!("Removing mod {}", modification);
    unimplemented!()
}
