mod embed;

pub fn get_version() -> String {
    embed::VERSION.into()
}

pub fn get_cert() -> String {
    embed::CERT.into()
}
