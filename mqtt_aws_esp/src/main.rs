use anyhow::Result;
use core::convert::TryInto;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::mqtt::client::*;
use esp_idf_svc::tls::X509;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use log::{error, info};
use std::{mem, slice, time::Duration};
// use embedded_svc::utils::mqtt::client::{ConnState};

const MQTT_CLIENT_ID: &str = "esp-mqtt-demo";
const MQTT_TOPIC: &str = "esp-mqtt-demo";

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> Result<()> {
    let ssid = include_str!("certificates/wifi_ssid.txt").trim_end();
    let pass = include_str!("certificates/wifi_pass.txt").trim_end();
    // let ssid: &str = include_str!("certificates/wifi_ssid.txt").trim_end();
    // let pass: &str = include_str!("certificates/wifi_pass.txt").trim_end();

    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: pass.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected");

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}

// Define a function to convert certificates to X509 format
// https://github.com/WoongH/esp32-rust-aws-iot-example/blob/main/src/main.rs
fn convert_certificate(mut certificate_bytes: Vec<u8>) -> X509<'static> {
    // Append NUL
    certificate_bytes.push(0);

    // Convert the certificate
    let certificate_slice: &[u8] = unsafe {
        let ptr: *const u8 = certificate_bytes.as_ptr();
        let len: usize = certificate_bytes.len();
        mem::forget(certificate_bytes);

        slice::from_raw_parts(ptr, len)
    };

    // Return the certificate file in the correct format
    X509::pem_until_nul(certificate_slice)
}

fn create_mqtt_client(
    url: &str,
    client_id: &str,
) -> Result<(EspMqttClient<'static>, EspMqttConnection)> {
    let server_cert_bytes: Vec<u8> = include_bytes!("certificates/AmazonRootCA1.pem").to_vec();
    let client_cert_bytes: Vec<u8> = include_bytes!("certificates/DeviceCertificate.pem").to_vec();
    let private_key_bytes: Vec<u8> = include_bytes!("certificates/client.private.key").to_vec();

    let server_cert: X509 = convert_certificate(server_cert_bytes);
    let client_cert: X509 = convert_certificate(client_cert_bytes);
    let private_key: X509 = convert_certificate(private_key_bytes);

    let (mqtt_client, mqtt_conn) = EspMqttClient::new(
        url,
        &MqttClientConfiguration {
            client_id: Some(client_id),
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
            server_certificate: Some(server_cert),
            client_certificate: Some(client_cert),
            private_key: Some(private_key),
            ..Default::default()
        },
    )?;

    Ok((mqtt_client, mqtt_conn))
}

fn run(client: &mut EspMqttClient, conn: &mut EspMqttConnection, topic: &str) -> Result<()> {
    std::thread::scope(|s| {
        info!("About to start the MQTT client");

        // Need to immediately start pumping the connection for messages, or else subscribe() and publish() below will not work
        // Note that when using the alternative constructor - `EspMqttClient::new_cb` - you don't need to
        // spawn a new thread, as the messages will be pumped with a backpressure into the callback you provide.
        // Yet, you still need to efficiently process each message in the callback without blocking for too long.
        //
        // Note also that if you go to http://tools.emqx.io/ and then connect and send a message to topic
        // "esp-mqtt-demo", the client configured here should receive it.
        std::thread::Builder::new()
            .stack_size(6000)
            .spawn_scoped(s, move || {
                info!("MQTT Listening for messages");

                while let Ok(event) = conn.next() {
                    info!("[Queue] Event: {}", event.payload());
                }

                info!("Connection closed");
            })
            .unwrap();

        loop {
            if let Err(e) = client.subscribe(topic, QoS::AtMostOnce) {
                error!("Failed to subscribe to topic \"{topic}\": {e}, retrying...");

                // Re-try in 0.5s
                std::thread::sleep(Duration::from_millis(500));

                continue;
            }

            info!("Subscribed to topic \"{topic}\"");

            // Just to give a chance of our connection to get even the first published message
            std::thread::sleep(Duration::from_millis(500));

            let payload = "Hello from esp-mqtt-demo!";

            loop {
                client.enqueue(topic, QoS::AtMostOnce, false, payload.as_bytes())?;

                info!("Published \"{payload}\" to topic \"{topic}\"");

                let sleep_secs = 2;

                info!("Now sleeping for {sleep_secs}s...");
                std::thread::sleep(Duration::from_secs(sleep_secs));
            }
        }
    })
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let endpoint = include_str!("certificates/endpoint.txt").trim_end();
    info!("{}", endpoint);

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;
    connect_wifi(&mut wifi)?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi DHCP info: {:?}", ip_info);

    let (mut client, mut conn) = create_mqtt_client(endpoint, MQTT_CLIENT_ID)?;

    run(&mut client, &mut conn, MQTT_TOPIC)?;

    // #[allow(unreachable_code)]
    Ok(())
}
