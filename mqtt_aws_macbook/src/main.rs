use anyhow::Result;
use rumqttc::{Client, Connection, MqttOptions, QoS, TlsConfiguration, Transport};
use std::{thread, time::Duration};

// Define a function to create an MQTT client and connection
fn create_mqtt_client(endpoint: &str) -> Result<(Client, Connection)> {
    let transport = {
        let ca = include_bytes!("certificates/AmazonRootCA1.pem").to_vec();
        let client_cert = include_bytes!("certificates/DeviceCertificate.pem").to_vec();
        let client_key = include_bytes!("certificates/client.private.key").to_vec();

        Transport::Tls(TlsConfiguration::Simple {
            ca,
            alpn: None,
            client_auth: Some((client_cert, client_key)),
        })
    };

    let (client, conn) = {
        let mut mqttoptions = MqttOptions::new("my-macbook-air", endpoint, 8883);
        mqttoptions.set_keep_alive(std::time::Duration::from_secs(5));
        mqttoptions.set_transport(transport);

        Client::new(mqttoptions, 10)
    };

    Ok((client, conn))
}

fn publish(client: Client) {
    thread::sleep(Duration::from_secs(1));
    client.subscribe("hello/+/world", QoS::AtMostOnce).unwrap();
    for i in 0..3_usize {
        let payload = vec![1; i];
        let topic = format!("hello/{i}/world");
        println!("{:?}", topic);
        let qos = QoS::AtLeastOnce;

        client.publish(topic, qos, true, payload).unwrap();
    }

    thread::sleep(Duration::from_secs(1));
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let aws_iot_endpoint: &str = std::str::from_utf8(include_bytes!("certificates/endpoint.txt"))?;
    println!("AWS_IOT_ENDPOINT: {:?}", aws_iot_endpoint);

    // Create an MQTT client and connection
    let (client, mut conn) = create_mqtt_client(aws_iot_endpoint)?;
    println!("Connected to AWS IoT Core");

    publish(client);
    println!("Published messages");

    for (i, notification) in conn.iter().enumerate() {
        match notification {
            Ok(notif) => {
                println!("{i}. Notification = {notif:?}");
            }
            Err(error) => {
                println!("{i}. Notification = {error:?}");
                return Ok(());
            }
        }
    }

    Ok(())
}
