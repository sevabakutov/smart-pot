mod private {
    use crate::{
        config::Settings,
        core::{Result, SmartPotError},
    };
    use log::info;
    use rumqttc::tokio_rustls::rustls::{ClientConfig, RootCertStore};
    use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS, TlsConfiguration, Transport};
    use std::{sync::Arc, time::Duration};
    use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};

    pub struct IoTHub {
        pub client: AsyncClient,
        settings: Settings,
    }

    impl IoTHub {
        pub fn from_settings(settings: Settings) -> Result<(Self, EventLoop)> {
            let id = settings.device().id().clone();
            let host = settings.hub().host().clone();
            let port = settings.hub().port();

            let mut options = MqttOptions::new(id, host, port);

            let tls_config = Self::create_tls_config()?;

            options.set_keep_alive(Duration::from_secs(10));
            options.set_transport(Transport::Tls(tls_config));

            let (client, eventloop) = AsyncClient::new(options, 10);

            Ok((Self { client, settings }, eventloop))
        }

        pub async fn send(&self, message: &str) -> Result<()> {
            let topic = format!("devices/{}/messages/events/", self.settings.device().id());
            info!("Sending: '{}' to '{}' endpoint", message, topic);

            self.client
                .publish(topic, QoS::AtLeastOnce, false, message)
                .await
                .map_err(Into::into)
        }

        fn create_tls_config() -> Result<TlsConfiguration> {
            let mut root_store = RootCertStore::empty();

            let cert_chain = CertificateDer::pem_file_iter("device-cert.pem")
                .map_err(|err| SmartPotError::TLSError(err.to_string()))?
                .flatten()
                .collect::<Vec<_>>();

            for der in cert_chain.iter() {
                root_store
                    .add(der.clone())
                    .map_err(|err| SmartPotError::TLSError(err.to_string()))?;
            }

            let key_der = PrivateKeyDer::from_pem_file("device.key")
                .map_err(|err| SmartPotError::TLSError(err.to_string()))?;

            let conf = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_client_auth_cert(cert_chain, key_der)
                .map_err(|err| SmartPotError::TLSError(err.to_string()))?;

            Ok(TlsConfiguration::Rustls(Arc::new(conf)))
        }
    }
}

crate::mod_interface! {
    orphan use {
        IoTHub
    };
}
