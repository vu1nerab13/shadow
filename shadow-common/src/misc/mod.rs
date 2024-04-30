mod version;

pub fn get_version() -> String {
    version::VERSION.into()
}
