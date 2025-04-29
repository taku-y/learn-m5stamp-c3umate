use esp_idf_hal::adc::oneshot::{AdcDriver, config::AdcChannelConfig, AdcChannelDriver};
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Take Peripherals
    let peripherals = Peripherals::take().unwrap();

    // Configure and Initialize ADC
    let mut adc = AdcDriver::new(peripherals.adc1).unwrap();
    let config = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };
    let mut adc_pin = AdcChannelDriver::new(&adc, peripherals.pins.gpio0, &config).unwrap();

    // Configure and Initialize LEDC Timer Driver
    let timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &TimerConfig::default()
            .frequency(50.Hz())
            .resolution(Resolution::Bits14),
    )
    .unwrap();

    // Configure and Initialize LEDC Driver
    let mut driver = LedcDriver::new(
        peripherals.ledc.channel0,
        timer_driver,
        peripherals.pins.gpio8,
    )
    .unwrap();

    // Get Max Duty
    let max_duty = driver.get_max_duty();
    let min_limit = max_duty * 5 / 10 / 20;
    let max_limit = max_duty * 24 / 10 / 20;
    log::info!("Max Duty: {}", max_duty);

    loop {
        // Read ADC value
        let adc_value = adc.read(&mut adc_pin).unwrap();
        // log::info!("ADC value: {}", adc_value);

        // Map ADC value (0-4095) to PWM duty (0-max_duty)
        let duty = map(adc_value.into(), 0, 2500, min_limit, max_limit);
        // let duty = map(adc_value.into(), 0, 4095, min_limit, max_limit);
        driver.set_duty(duty).unwrap();
        log::info!("(adc, duty): {:?}", (adc_value, duty));

        FreeRtos::delay_ms(20);
    }
}

// Function that maps one range to another
fn map(x: u32, in_min: u32, in_max: u32, out_min: u32, out_max: u32) -> u32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}
