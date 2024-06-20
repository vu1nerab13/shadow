mod embed;
mod sender;

pub use sender::SenderSink;

pub fn get_version() -> String {
    embed::VERSION.into()
}
