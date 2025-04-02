mod private {
    use base64::prelude::*;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use urlencoding::encode;

    const AZURE_IOT_CA_CERT: &[u8] = include_bytes!("../../../DigiCertGlobalRootG2.pem");
    
    // /// Generate an Azure IoT SAS token
    // pub fn generate_sas_token(
    //     hub_name: &str,
    //     device_id: &str,
    //     key: &str,
    //     expiry_unix_ts: u64,
    // ) -> String {
    //     let sr = format!("{hub_name}.azure-devices.net/devices/{device_id}");

    //     let to_sign = format!("{sr}\n{expiry_unix_ts}");

    //     let key_bytes = BASE64_STANDARD.decode(key).unwrap();

    //     let mut mac = Hmac::<Sha256>::new_from_slice(&key_bytes).unwrap();
    //     mac.update(to_sign.as_bytes());
    //     let signature = mac.finalize().into_bytes();

    //     let sig_base64 = BASE64_STANDARD.encode(signature);

    //     format!(
    //         "SharedAccessSignature sr={}&sig={}&se={}",
    //         encode(&sr),
    //         encode(&sig_base64),
    //         expiry_unix_ts
    //     )
    // }

    pub fn generate_sas_token_dps(
        id_scope: &str,
        registration_id: &str,
        key: &str,
        expiry_unix_ts: u64
    ) -> String {
        let sr = format!("{id_scope}/registrations/{registration_id}");

        let to_sign = format!("{sr}\n{expiry_unix_ts}");

        let key_bytes = BASE64_STANDARD.decode(key).expect("invalid base64 in key");

        let mut mac = Hmac::<Sha256>::new_from_slice(&key_bytes)
            .expect("HMAC can take key of any size");

        mac.update(to_sign.as_bytes());
        let signature = mac.finalize().into_bytes();

        let sig_base64 = BASE64_STANDARD.encode(signature);

        format!(
            "SharedAccessSignature sr={}&sig={}&se={}&skn=registration",
            encode(&sr),
            encode(&sig_base64),
            expiry_unix_ts
        )
    }

    pub fn set_global_ca_store() -> Result<()> {
        let status_code = unsafe { esp_tls_init_global_ca_store() };

        if status_code != ESP_OK {
            return Err(SmartPotError::CAError(
                "Failed to initialize ca store. Status code: {status_code}".to_string(),
            ));
        }
        let cstr = std::ffi::CString::new(AZURE_IOT_CA_CERT)
            .expect("Certificate string contained internal NUL byte");

        let status_code = unsafe {
            esp_tls_set_global_ca_store(
                cstr.as_ptr() as *const u8,
                cstr.as_bytes_with_nul().len() as u32,
            )
        };

        if status_code != ESP_OK {
            return Err(SmartPotError::CAError(format!(
                "failed to set global ca store (err={})",
                status_code
            )));
        };

        Ok(())
    }
}

crate::mod_interface!{
    layer iot_hub;
    layer dps;

    own use {
        // generate_sas_token,
        generate_sas_token_dps,
        set_global_ca_store
    };
}