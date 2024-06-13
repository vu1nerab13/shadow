mod misc;
pub mod network;
pub mod web;

use clap::Parser;

/// Server command line arguments
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "A high performance rat server and client written in 100% safe rust"
)]
pub struct AppArgs {
    #[arg(short, long, default_value_t = String::from("debug"))]
    pub verbose: String,

    #[arg(short, long, default_value_t = String::from("0.0.0.0:1244"))]
    pub server_addr: String,

    #[arg(short, long, default_value_t = String::from("0.0.0.0:9000"))]
    pub web_addr: String,
}
