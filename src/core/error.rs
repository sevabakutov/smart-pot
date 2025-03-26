mod private {
    use config::ConfigError;
    use rumqttc::ClientError;
    use thiserror::Error;

    /// # SmartPotError
    #[derive(Error, Debug)]
    pub enum SmartPotError {
        #[error("Config error:\n{0}")]
        ConfigError(#[from] ConfigError),

        #[error("Client error:\n{0}")]
        ClientError(#[from] ClientError),

        #[error("IO error:\n{0}")]
        IoError(#[from] std::io::Error),

        #[error("TLS error:\n{0}")]
        TLSError(String),
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
