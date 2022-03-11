use mqtt::Message;
use paho_mqtt as mqtt;
use std::env::VarError;
use std::sync::mpsc::Receiver;
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
        let client_opt = mqtt::CreateOptionsBuilder::new()
            .mqtt_version(mqtt::MQTT_VERSION_5)
            .server_uri(connection_str)
            .finalize();

        let client = mqtt::client::Client::new(client_opt).unwrap();

        client.connect(
            mqtt::ConnectOptionsBuilder::new()
                .mqtt_version(mqtt::MQTT_VERSION_5)
                .user_name(username)
                .password(password)
                //.keep_alive_interval(Duration::from_secs(20))
                //.automatic_reconnect(Duration::from_secs(1), Duration::from_secs(8192))
                //.clean_start(false)
                .connect_timeout(Duration::from_secs(5))
                .finalize(),
        )?;

        Ok(Mqtt {
            client,
            topic_prefix,
        })
    }

    pub fn reconnect(&mut self) {
        println!("Reconnecting");
        if let Err(msg) = self.client.reconnect() {
            println!("Reconnect fail {}, sleep 5s and try", msg.to_string());
            std::thread::sleep(Duration::from_secs(5));
            self.reconnect();
        } else {
            println!("Reconnected");
        }
    }

    pub fn consume(&mut self) -> Receiver<Option<Message>> {
        self.client.start_consuming()
    }

    pub fn publish(&mut self, topic: &str, message: String) {
        let topic = format!("{}/{}", self.topic_prefix, topic);
        let _ = self
            .client
            .publish(mqtt::Message::new_retained(topic, message, 0));
    }

    pub fn subscribe(&mut self, topic: &str) {
        let topic = format!("{}/{}", self.topic_prefix, topic);
        let _ = self.client.subscribe(&topic, 1);
    }
}
