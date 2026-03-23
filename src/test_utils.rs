//! Test utilities.

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    /// A global lock for environment variable modifications in tests.
    pub static ref ENV_LOCK: Mutex<()> = Mutex::new(());
}
