//!
//! File with constants.
//!

mod private {
    pub static ACCESS_KEY: &str = "TheAccessKey";
}

crate::mod_interface! {
    own use {
        ACCESS_KEY
    };
}
