mod embed;
pub mod sender;

pub fn get_version() -> String {
    embed::VERSION.into()
}
