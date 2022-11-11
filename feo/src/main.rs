use clap::Parser;
use std::{io::Error, process};

#[derive(Parser, Debug)]
#[clap(author = "Victor Nilsson (github.com/vcrn)", version, about = "Simple system resource monitoring CLI tool for Linux systems, with GPU temperature monitoring for Raspberry Pi)", long_about = None)]
pub struct Args {
    /// Select color-scheme for monitor: 'w' for white, 'b' for black, 's' for standard
    #[clap(short = 'c', long, default_value = "s")]
    color: char,

    /// Monitor the GPU temperature (only available for Raspberry Pi)
    #[clap(short, long)]
    gpu: bool,

    /// Set the delay between updates, in whole seconds
    #[clap(short, long, default_value_t = 2)]
    delay: usize,
}

fn terminate_with_error(err: Error) {
    eprintln!("Compability issue. FeO is designed to run on Linux. GPU temperature monitor option only works for Raspberry Pi. Error message: {}", err);
    process::exit(1);
}

fn main() {
    let args = Args::parse();

    if let Err(e) = feo::run(args.delay, args.gpu, args.color) {
        terminate_with_error(e);
    }
}
