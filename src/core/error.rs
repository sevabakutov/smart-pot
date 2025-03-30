mod private {
    use config::ConfigError;
    use esp_idf_sys::EspError;
    //use rumqttc::ClientError;
    use thiserror::Error;

    /// # SmartPotError
    #[derive(Error, Debug)]
    pub enum SmartPotError {
        #[error("Config error:\n{0}")]
        ConfigError(#[from] ConfigError),

        #[error("IO error:\n{0}")]
        IoError(#[from] std::io::Error),

        #[error("TLS error:\n{0}")]
        TLSError(String),

        #[error("Esp error:\n{0}")]
        EspError(#[from] EspError),

        #[error("OneWire error:\n{0}")]
        OneWireError(String),

        #[error("{0}")]
        PrsingError(String)
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
