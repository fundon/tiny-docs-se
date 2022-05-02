use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build sqlite db indexes from markdown files
    Build {
        #[clap(short, long)]
        #[clap(parse(try_from_str))]
        #[clap(help("path"))]
        path: PathBuf,

        #[clap(short, long)]
        #[clap(parse(try_from_str))]
        #[clap(help("locale"))]
        locale: String,

        #[clap(short, long)]
        #[clap(parse(try_from_str))]
        #[clap(help("version"))]
        version: String,
    },
    /// Run a search server for web
    Server {
        #[clap(short, long)]
        #[clap(parse(try_from_str))]
        #[clap(default_value_t = 3030)]
        #[clap(help("port"))]
        port: u16,
    },
}

mod cmd;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .map_err(|e| anyhow::anyhow!(e))?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            path,
            locale,
            version,
        } => cmd::build::execute(path, locale, version),
        Commands::Server { port } => {
            use tokio::runtime::Builder;
            let rt = Builder::new_multi_thread().enable_all().build()?;
            rt.block_on(cmd::server::execute(port))
        }
    }
}
