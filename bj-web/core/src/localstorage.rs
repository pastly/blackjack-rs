use serde::{Deserialize, Serialize};
use web_sys::Storage;

pub mod lskeys {
    pub const LS_KEY_BS_CARD: &str = "bj-current-bs-card";
}

pub struct LSVal<T>
where
    T: Serialize,
{
    key: String,
    val: T,
    is_session: bool,
}

impl<T> LSVal<T>
where
    T: Serialize,
    for<'de> T: Deserialize<'de>,
{
    pub fn from_ls_or_default(is_session: bool, key: &str, def: T) -> Self {
        match ls_get(is_session, key) {
            None => {
                ls_set(is_session, key, &def);
                Self {
                    key: key.to_owned(),
                    val: def,
                    is_session,
                }
            }
            Some(v) => Self {
                key: key.to_owned(),
                val: v,
                is_session,
            },
        }
    }

    pub fn from_ls(is_session: bool, key: &str) -> Option<Self> {
        if let Some(v) = ls_get(is_session, key) {
            Some(Self {
                key: key.to_owned(),
                val: v,
                is_session,
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
        ls_set(self.is_session, &self.key, &self.val);
    }
}

fn ls(is_session: bool) -> Storage {
    let win = web_sys::window().expect("should have a window in this context");
    if is_session {
        win.session_storage()
            .expect("Err getting session_storage")
            .expect("None getting session_storage")
    } else {
        win.local_storage()
            .expect("Err getting local_storage")
            .expect("None getting local_storage")
    }
}

fn ls_get<T>(is_session: bool, key: &str) -> Option<T>
where
    for<'de> T: Deserialize<'de>,
{
    if let Some(val) = ls(is_session)
        .get(key)
        .expect("Err getting from local storage")
    {
        serde_json::from_str(&val).ok()
    } else {
        None
    }
}

fn ls_set<T>(is_session: bool, key: &str, val: &T)
where
    T: Serialize,
{
    let val = serde_json::to_string(&val).unwrap();
    ls(is_session).set(key, &val).unwrap()
}
