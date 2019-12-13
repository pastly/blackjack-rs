use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

pub fn write_maybexz<T>(fd: impl Write, data: &T, xz: bool) -> Result<(), serde_json::error::Error>
where
    T: Serialize,
{
    if xz {
        serde_json::to_writer(XzEncoder::new(fd, 9), &data)
    } else {
        serde_json::to_writer(fd, &data)
    }
}

pub fn read_maybexz<T>(fd: impl Read, xz: bool) -> Result<T, serde_json::error::Error>
where
    for<'de> T: Deserialize<'de>,
{
    if xz {
        serde_json::from_reader(XzDecoder::new(fd))
    } else {
        serde_json::from_reader(fd)
    }
}
