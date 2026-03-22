use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref ENV_LOCK: Mutex<()> = Mutex::new(());
}
