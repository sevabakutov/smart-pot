
mod private {
    use std::env::VarError;
    use thiserror::Error;

    /// # SmartPotError
    #[derive(Error, Debug)]
    pub enum SmartPotError {
        #[error("Initialization error:\n{0}")]
        InitializationError(String),
        #[error("Reading enviroment variable error: \n{0}")]
        EnvError(#[from] VarError)
    }

    /// Shortcut for std::result::Result<T, SmartPotError>
    pub type Result<T> = std::result::Result<T, SmartPotError>;
}

crate::mod_interface! {
    
}