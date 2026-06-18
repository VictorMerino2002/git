mod commands;
mod config;
mod index;
mod objects;
mod repository;
mod utils;

use clap::{Parser, Subcommand};

use crate::commands::{
    CatFileCommand, CheckoutCommand, HashObjectCommand, InitCommand, LogCommand, LsFilesCommand,
    LsTreeCommand, RevParseCommand, ShowRefCommand, TagCommand,
};

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
    Log {
        #[arg(default_value = "HEAD")]
        commit: String,
    },
    LsTree {
        #[arg(short = 'r', value_name = "recursive")]
        recursive: bool,
        #[arg(default_value = "HEAD")]
        tree: String,
    },
    Checkout {
        #[arg()]
        commit: String,
        #[arg()]
        path: String,
    },
    ShowRef,
    Tag {
        #[arg(short = 'a')]
        annotation: bool,
        #[arg()]
        name: Option<String>,
        #[arg(default_value = "HEAD")]
        object: String,
    },
    RevParse {
        #[arg(short = 't')]
        object_type: String,
        #[arg()]
        name: String,
    },
    LsFiles {
        #[arg(long = "verbose")]
        verbose: bool,
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
        Commands::Log { commit } => LogCommand::new(&commit).and_then(|cmd| cmd.execute()),
        Commands::LsTree { recursive, tree } => LsTreeCommand::new(&tree, recursive).execute(),
        Commands::Checkout { commit, path } => CheckoutCommand::new(&commit, &path).execute(),
        Commands::ShowRef => ShowRefCommand::execute(),
        Commands::Tag {
            annotation,
            name,
            object,
        } => TagCommand::new(annotation, &name, &object).execute(),
        Commands::RevParse { object_type, name } => {
            RevParseCommand::new(&object_type, &name).and_then(|cmd| cmd.execute())
        }
        Commands::LsFiles { verbose } => LsFilesCommand::new(verbose).execute(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
