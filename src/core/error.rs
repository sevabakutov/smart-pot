//!
//! Error Handling Module
//!

mod private {
    use esp_idf_sys::EspError;
    use thiserror::Error;

    /// # SmartPotError
    ///
    /// Enum representing all possible errors in the SmartPot system. Each variant corresponds to a specific type of error,
    /// such as IO errors, hardware communication issues, or data parsing errors.
    #[derive(Error, Debug)]
    pub enum SmartPotError {
        #[error("IO error:\n{0}")]
        IoError(#[from] std::io::Error),

        #[error("CA error:\n{0}")]
        CAError(String),

        #[error("Esp error:\n{0}")]
        EspError(#[from] EspError),

        #[error("OneWire error:\n{0}")]
        OneWireError(String),

        #[error("I2C error:\n{0}")]
        I2cError(String),

        #[error("Dht error:\n{0}")]
        DhtError(String),

        #[error("{0}")]
        ParsingError(String),
    }

    /// Shortcut for std::result::Result<T, SmartPotError>
    pub type Result<T> = std::result::Result<T, SmartPotError>;
}

crate::mod_interface! {
    orphan use {
        SmartPotError,
        Result
    };
}
