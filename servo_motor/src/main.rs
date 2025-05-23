use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

fn main() {
    esp_idf_sys::link_patches();

    // Take Peripherals
    let peripherals = Peripherals::take().unwrap();

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

    // Get Max Duty and Calculate Upper and Lower Limits for Servo
    let max_duty = driver.get_max_duty();
    println!("Max Duty {}", max_duty);
    let min_limit = max_duty * 5 / 10 / 20;
    println!("Min Limit {}", min_limit);
    let max_limit = max_duty * 24 / 10 / 20;
    println!("Max Limit {}", max_limit);

    // Define Starting Position
    driver
        .set_duty(map(0, 0, 180, min_limit, max_limit))
        .unwrap();
    // Give servo some time to update
    FreeRtos::delay_ms(500);

    loop {
        // Sweep from 0 degrees to 180 degrees
        for angle in 0..180 {
            // Print Current Angle for visual verification
            println!("Current Angle {} Degrees", angle);
            // Set the desired duty cycle
            driver
                .set_duty(map(angle, 0, 180, min_limit, max_limit))
                .unwrap();
            // Give servo some time to update
            FreeRtos::delay_ms(50);
        }

        // Sweep from 180 degrees to 0 degrees
        for angle in (0..180).rev() {
            // Print Current Angle for visual verification
            println!("Current Angle {} Degrees", angle);
            // Set the desired duty cycle
            driver
                .set_duty(map(angle, 0, 180, min_limit, max_limit))
                .unwrap();
            // Give servo some time to update
            FreeRtos::delay_ms(50);
        }
    }
}

// Function that maps one range to another
fn map(x: u32, in_min: u32, in_max: u32, out_min: u32, out_max: u32) -> u32 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}