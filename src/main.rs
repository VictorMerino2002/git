mod commands;
mod config;
mod objects;
mod repository;
mod sha1;
mod zlib;

use clap::{Parser, Subcommand};

use crate::commands::{CatFileCommand, HashObjectCommand, InitCommand};

#[derive(Parser)]
#[command(name = "git")]
#[command(about = "A simple implementation of git in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(default_value = ".")]
        path: String,
    },
    CatFile {
        object_type: String,
        object: String,
    },

    HashObject {
        #[arg(short = 't', value_name = "type", default_value = "blob")]
        object_type: String,
        #[arg(short = 'w')]
        write: bool,
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { path } => InitCommand::new(path).execute(),
        Commands::CatFile {
            object_type,
            object,
        } => CatFileCommand::new(&object_type, &object).and_then(|cmd| cmd.execute()),
        Commands::HashObject {
            object_type,
            write,
            path,
        } => HashObjectCommand::new(&object_type, write, &path).and_then(|cmd| cmd.execute()),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
