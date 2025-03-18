//!
//! File with core concepts of `Smart Pot`
//! 


use std::env::VarError;

use azure_iot_sdk::{DeviceKeyTokenSource, IoTHubClient};
use thiserror::Error;

use crate::*;
use constants::ACCESS_KEY;
use telemtry::Telemetry;

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

#[derive(Debug)]
pub struct IoTHub {
    client: IoTHubClient
}

impl IoTHub {
    /// Gets already registered device and returns IoTHub.
    pub async fn new() -> Result<Self> {
        let iothub_hostname = std::env::var("IOTHUB_HOSTNAME")?;
        let device_id = std::env::var("DEVICE_ID")?;
        let token_source = DeviceKeyTokenSource::new(
            iothub_hostname.as_str(),
            device_id.as_str(),
            ACCESS_KEY,
        ).unwrap();

        let client = IoTHubClient::new(&iothub_hostname, device_id, token_source)
            .await
            .map_err(|error| SmartPotError::InitializationError(error.to_string()))?;

        Ok(Self { client })
    }

    /// Register device and returns IoTHub.
    pub async fn register_device() -> Result<Self> {
        let scope_id = std::env::var("DPS_SCOPE_ID")?;
        let registration_id = std::env::var("DPS_REGISTRATION_ID")?;
        let device_key = std::env::var("DPS_DEVICE_KEY")?;
    
        let client = IoTHubClient::from_provision_service(&scope_id, registration_id, &device_key, 5)
            .await
            .map_err(|error| SmartPotError::InitializationError(error.to_string()))?;

        Ok(Self { client })
    }

    pub async fn get_telemtry(&self) -> Result<Telemetry> {
        unimplemented!()
    }

    pub async fn send_telemetry(&self) -> Result<()> {
        unimplemented!()
    }
}