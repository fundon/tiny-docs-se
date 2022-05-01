use std::path::PathBuf;

use clap::{AppSettings, Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
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
        path: PathBuf,
    },
    /// Run a search server for web
    Server {
        #[clap(short, long)]
        #[clap(parse(try_from_str))]
        #[clap(default_value_t = 3030)]
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

    match &cli.command {
        Commands::Build { path } => cmd::build::execute(path.to_path_buf()),
        Commands::Server { port } => {
            use tokio::runtime::Builder;
            let rt = Builder::new_multi_thread().enable_all().build()?;
            rt.block_on(cmd::server::execute(*port))
        }
    }
}
