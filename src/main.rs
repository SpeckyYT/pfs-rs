#![feature(iter_next_chunk)]

mod artemis;
mod pack;
mod unpack;
mod xor;

use std::{path::PathBuf, fs};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    pub path: PathBuf,
    #[arg(long, short)]
    pub output: Option<PathBuf>,
}

fn main() {
    let options = Cli::parse();

    if options.path.is_dir() && !cfg!(debug_assertions) {
        panic!("packing doesn't work correctly");
    }

    match options.path {
        file if file.is_file() => {
            let folder = options.output.unwrap_or(
                file.with_file_name(format!("{}-pfs", file.file_name().unwrap().to_str().unwrap()))
            );

            fs::create_dir_all(&folder).expect("failed creating folder");

            unpack::unpack(&file, &folder);
        },
        dir if dir.is_dir() => {
            let file = options.output.unwrap_or(
                dir.with_file_name(format!("{}.pfs", dir.file_name().unwrap().to_str().unwrap()))
            );

            pack::pack(&dir, &file);
        },
        path => panic!("path `{path:?}` is invalid"),
    }
}
