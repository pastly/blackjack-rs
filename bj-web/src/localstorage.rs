use serde::{Deserialize, Serialize};
use web_sys::Storage;

fn ls() -> Storage {
    let win = web_sys::window().expect("should have a window in this context");
    win.local_storage()
        .expect("Err getting local_storage")
        .expect("None getting local_storage")
}

pub fn ls_get<T>(key: &str) -> Option<T>
where
    for<'de> T: Deserialize<'de>,
{
    if let Some(val) = ls().get(key).expect("Err getting from local storage") {
        serde_json::from_str(&val).ok()
    } else {
        None
    }
}

pub fn ls_set<T>(key: &str, val: &T) -> ()
where
    T: Serialize,
{
    let val = serde_json::to_string(&val).unwrap();
    ls().set(key, &val).unwrap()
}
