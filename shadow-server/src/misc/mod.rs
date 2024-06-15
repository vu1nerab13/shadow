mod embed;

pub fn get_version() -> String {
    embed::VERSION.into()
}
