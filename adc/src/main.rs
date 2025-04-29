use esp_idf_hal::adc::config::Config;
use esp_idf_hal::adc::AdcChannelDriver;
use esp_idf_hal::adc::AdcDriver;
use esp_idf_hal::adc::*;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;

use esp_idf_hal::gpio::Gpio0;

fn main() { //  -> anyhow::Result<()> {
    let peripherals = Peripherals::take().unwrap();

    let mut adc = AdcDriver::new(peripherals.adc1, &Config::new().calibration(true)).unwrap();

    let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<{ attenuation::NONE }, _> =
        AdcChannelDriver::new(peripherals.pins.gpio0).unwrap();

    loop {
        let a = adc.read(&mut adc_pin).unwrap();

        // if a > 1700 || a < 1600 {
        //     println!("ADC value: {}", a);
        // }
        println!("ADC value: {}", a);
        FreeRtos::delay_ms(100);
    }
}
