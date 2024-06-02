use crabgrab::capture_stream::CapturePixelFormat;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S>(v: &CapturePixelFormat, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use CapturePixelFormat::*;

    let v: u8 = match v {
        Bgra8888 => 1,
        Argb2101010 => 2,
        V420 => 3,
        F420 => 4,
        _ => unreachable!(),
    };

    v.serialize(s)
}

pub fn deserialize<'de, D>(d: D) -> Result<CapturePixelFormat, D::Error>
where
    D: Deserializer<'de>,
{
    use CapturePixelFormat::*;

    match u8::deserialize(d)? {
        1 => Ok(Bgra8888),
        2 => Ok(Argb2101010),
        3 => Ok(V420),
        4 => Ok(F420),
        o => Err(D::Error::custom(format_args!(
            "invalid pixel format value {}",
            o
        ))),
    }
}
