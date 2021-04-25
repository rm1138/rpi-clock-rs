use paho_mqtt as mqtt;
use std::env::VarError;
use std::error::Error;
use std::time::Duration;

pub struct Mqtt {
    client: mqtt::Client,
    topic_prefix: String,
}

#[derive(Debug)]
pub struct MqttError {
    msg: String,
}

impl From<VarError> for MqttError {
    fn from(err: VarError) -> MqttError {
        MqttError {
            msg: err.to_string(),
        }
    }
}

impl From<mqtt::Error> for MqttError {
    fn from(err: mqtt::Error) -> MqttError {
        MqttError {
            msg: format!("{:?}", err),
        }
    }
}

impl Mqtt {
    pub fn connect() -> Result<Mqtt, MqttError> {
        let host = std::env::var("MQTT_HOST")?;
        let port = std::env::var("MQTT_PORT")?;
        let username = std::env::var("MQTT_USERNAME")?;
        let password = std::env::var("MQTT_PASSWORD")?;
        let topic_prefix = std::env::var("MQTT_TOPIC_PREFIX")?;

        let connection_str = format!("tcp://{}:{}", host, port);
        let client = mqtt::client::Client::new(connection_str).unwrap();

        client.connect(
            mqtt::ConnectOptionsBuilder::new()
                .user_name(username)
                .password(password)
                .automatic_reconnect(Duration::from_secs(5), Duration::from_secs(30))
                .finalize(),
        )?;

        Ok(Mqtt {
            client,
            topic_prefix,
        })
    }

    pub fn publish(&mut self, topic: &str, message: String) {
        let topic = format!("{}/{}", self.topic_prefix, topic);
        self.client
            .publish(mqtt::Message::new_retained(topic, message, 0));
    }
}
