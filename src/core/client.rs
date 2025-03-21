
mod private {
    use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
    use crate::{config::Settings, core::Result};

    pub struct IoTHub {
        pub client: AsyncClient,
        eventloop: EventLoop,
        settings: Settings
    }

    impl IoTHub {
        pub fn from_settings(settings: Settings) -> Self {
            let id = settings.device().id().clone();
            let host = settings.hub().host().clone();
            let port = settings.hub().port();

            let options = MqttOptions::new(id, host, port);
            let (client, eventloop) = AsyncClient::new(options, 10);

            Self { client, eventloop, settings: settings }
        }

        pub fn eventloop(&mut self) -> &mut EventLoop {
            &mut self.eventloop
        }

        pub async fn send(&mut self, message: &str) -> Result<()> {
            let topic = format!("devices/{}/messages/events/", self.settings.device().id());
            
            self.client
                .publish(topic, QoS::AtLeastOnce, false, message)
                .await
                .map_err(Into::into)
        }
    }
}

crate::mod_interface! {
    orphan use {
        IoTHub
    };
}