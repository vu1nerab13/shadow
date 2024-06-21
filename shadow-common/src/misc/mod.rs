mod embed;
mod sender;

pub use sender::transfer;

pub fn get_version() -> String {
    embed::VERSION.into()
}
