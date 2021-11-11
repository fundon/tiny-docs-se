use clap::{crate_authors, crate_description, crate_version, App, crate_name};

const VERSION: &str = concat!("v", crate_version!());

fn main() {
    let matches = App::new(crate_name!())
        .version(VERSION)
        .author(crate_authors!())
        .about(crate_description!())
        // .arg("-c, --config=[FILE] 'Sets a custom config file'")
        .subcommand(
            App::new("build")
                .about("build sqlite db indexes from md files")
                // .arg("-d, --debug 'Print debug information'"),
        )
        .subcommand(
            App::new("server")
                .about("run a search server for web")
                // .arg("-d, --debug 'Print debug information'"),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("build", m)) => {
            println!("build {:?}", m);
        }
        Some(("server", m)) => {
            println!("server {:?}", m);
        }
        _ => {
        }
    }
}
