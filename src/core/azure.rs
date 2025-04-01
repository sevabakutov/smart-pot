mod private {
    use base64::prelude::*;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use urlencoding::encode;

    /// Generate an Azure IoT SAS token
    pub fn generate_sas_token(
        hub_name: &str,
        device_id: &str,
        key: &str,
        expiry_unix_ts: u64,
    ) -> String {
        let sr = format!("{hub_name}.azure-devices.net/devices/{device_id}");

        let to_sign = format!("{sr}\n{expiry_unix_ts}");

        let key_bytes = BASE64_STANDARD.decode(key).unwrap();

        let mut mac = Hmac::<Sha256>::new_from_slice(&key_bytes).unwrap();
        mac.update(to_sign.as_bytes());
        let signature = mac.finalize().into_bytes();

        let sig_base64 = BASE64_STANDARD.encode(signature);

        format!(
            "SharedAccessSignature sr={}&sig={}&se={}",
            encode(&sr),
            encode(&sig_base64),
            expiry_unix_ts
        )
    }
}

crate::mod_interface!{
    layer iot_hub;

    own use {
        generate_sas_token
    };
}