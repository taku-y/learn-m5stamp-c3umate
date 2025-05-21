use anyhow::anyhow;
use log::*;
use std::thread;
use std::time::Duration;
use esp_idf_hal::{delay::FreeRtos, i2c::*, peripherals::Peripherals, prelude::*};
use as5600::As5600;

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().expect("never fail");

    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio0;
    let scl = peripherals.pins.gpio1;

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    let mut delay = FreeRtos;

    // initialize the BME280 using the primary I2C address 0x76
    let mut as5600 = As5600::new(i2c);
    let config = as5600.config().unwrap();
    println!("{:?}", config);

    thread::sleep(Duration::from_secs(2));

    let status = as5600.magnet_status().unwrap();
    let agc = as5600.automatic_gain_control().unwrap();
    let mag = as5600.magnitude().unwrap();
    let zmco = as5600.zmco().unwrap();

    println!("{:?}", status);
    println!("{:?}", agc);
    println!("{:?}", mag);
    println!("{:?}", zmco);

    thread::sleep(Duration::from_secs(2));

    loop {
        let value = as5600.angle().unwrap();
        println!("{:?}", value);
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}