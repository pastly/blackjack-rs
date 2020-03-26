use serde::{Deserialize, Serialize};
use web_sys::Storage;

pub struct LSVal<T>
where
    T: Serialize,
{
    key: String,
    val: T,
}

impl<T> LSVal<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    pub fn from_ls_or_default(key: &str, def: T) -> Self {
        match ls_get(key) {
            None => {
                ls_set(key, &def);
                Self {
                    key: key.to_owned(),
                    val: def,
                }
            }
            Some(v) => Self {
                key: key.to_owned(),
                val: v,
            },
        }
    }

    pub fn from_ls(key: &str) -> Option<Self> {
        if let Some(v) = ls_get(key) {
            Some(Self {
                key: key.to_owned(),
                val: v,
            })
        } else {
            None
        }
    }

    pub fn swap(&mut self, mut val: T) -> T {
        std::mem::swap(&mut self.val, &mut val);
        val
    }
}

impl<T> std::ops::Deref for LSVal<T>
where
    T: Serialize,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl<T> std::ops::DerefMut for LSVal<T>
where
    T: Serialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<T> std::ops::Drop for LSVal<T>
where
    T: Serialize,
{
    fn drop(&mut self) {
        ls_set(&self.key, &self.val);
    }
}

fn ls() -> Storage {
    let win = web_sys::window().expect("should have a window in this context");
    win.local_storage()
        .expect("Err getting local_storage")
        .expect("None getting local_storage")
}

fn ls_get<T>(key: &str) -> Option<T>
where
    for<'de> T: Deserialize<'de>,
{
    if let Some(val) = ls().get(key).expect("Err getting from local storage") {
        serde_json::from_str(&val).ok()
    } else {
        None
    }
}

fn ls_set<T>(key: &str, val: &T)
where
    T: Serialize,
{
    let val = serde_json::to_string(&val).unwrap();
    ls().set(key, &val).unwrap()
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
    const LS_KEY: &str = "ThisIsALongKeyStringBecauseShortOnesDontWork";
    #[wasm_bindgen_test]
    fn from_ls_doesnt_exist() {
        // fetching an item that doesn't exist returns None
        let val: Option<LSVal<()>> = LSVal::from_ls(LS_KEY);
        assert!(val.is_none());
    }

    #[wasm_bindgen_test]
    fn from_ls_does_exist() {
        // fetching an item that does exist returns it. Even if the scope in which the item was
        // inserted is gone
        {
            let val = LSVal::from_ls_or_default(LS_KEY, 1u8);
            assert_eq!(*val, 1);
        }
        let val: LSVal<u8> = LSVal::from_ls(LS_KEY).unwrap();
        assert_eq!(*val, 1);
    }

    #[wasm_bindgen_test]
    fn from_ls_or_default_doesnt_overwrite() {
        // when using the or_default variant, it doesn't overwrite the existing value
        {
            // set an initial value
            let val = LSVal::from_ls_or_default(LS_KEY, 1u8);
            assert_eq!(*val, 1);
        }
        // fetch again, but make sure we don't overwrite the existing value
        let val = LSVal::from_ls_or_default(LS_KEY, 2u8);
        assert_eq!(*val, 1);
    }
}
