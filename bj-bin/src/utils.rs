use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
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

/// Create the given file if it doesn't already exist. If it needs to be created, fill it with the
/// given serializable data. Otherwise don't use the given data at all. Bubbles up any file system
/// errors (other than the error of "already exists." Panics if unable to serialize/write the data
/// to the file.
pub fn create_if_not_exist<T>(fname: &str, data: &T) -> Result<(), Box<dyn std::error::Error>>
where
    T: Serialize,
{
    match OpenOptions::new().create_new(true).write(true).open(fname) {
        Ok(fd) => {
            // able to create the file, so we need to fill it
            println!("Creating and filling {}", fname);
            match write_maybexz(fd, data, fname.ends_with(".xz")) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => {
            // unable to create the file, and that might be because it already exists.
            // ignore errors from it already existing, but bubble up all others
            match e.kind() {
                io::ErrorKind::AlreadyExists => Ok(()),
                _ => Err(e.into()),
            }
        }
    }
}
