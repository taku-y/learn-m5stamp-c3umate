mod buttons;
mod env;
mod evaluator;
mod sin_policy;

use anyhow::Result;
use as5600::As5600;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use esp_idf_svc::hal::i2c::*;
use esp_idf_svc::hal::ledc::{
    config::TimerConfig, LedcChannel, LedcDriver, LedcTimer, LedcTimerDriver, Resolution,
};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;

use buttons::Buttons;
use env::PendulumEnv;
use evaluator::PendulumEvaluator;
use sin_policy::SinPolicy;
use std::sync::atomic::{AtomicU8, Ordering};

static STATE: AtomicU8 = AtomicU8::new(0);

fn polling(env: &mut PendulumEnv, evaluator: &mut PendulumEvaluator, policy1: &mut SinPolicy) {
    log::info!("polling: {}", STATE.load(Ordering::Relaxed));
    match STATE.load(Ordering::Relaxed) {
        // Idle
        0 => FreeRtos::delay_ms(50),

        // Run an episode
        1 => evaluator.evaluate(policy1, env, 0).unwrap(),

        // Send episode data to the server
        2 => {
            FreeRtos::delay_ms(1000);
            STATE.store(0, Ordering::Relaxed);
        }

        // Receive model parameters from the server
        3 => {
            FreeRtos::delay_ms(1000);
            STATE.store(0, Ordering::Relaxed);
        }

        // Clear the episode data
        4 => {
            FreeRtos::delay_ms(1000);
            STATE.store(0, Ordering::Relaxed);
        }
        _ => {}
    }
}

fn create_as5600<'d>(
    i2c: I2C0,
    sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
) -> Result<As5600<I2cDriver<'d>>> {
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;

    let mut as5600 = As5600::new(i2c_driver);

    let config = as5600.config().unwrap();
    println!("{:?}", config);

    FreeRtos::delay_ms(2000);

    let status = as5600.magnet_status().unwrap();
    let agc = as5600.automatic_gain_control().unwrap();
    let mag = as5600.magnitude().unwrap();
    let zmco = as5600.zmco().unwrap();

    println!("{:?}", status);
    println!("{:?}", agc);
    println!("{:?}", mag);
    println!("{:?}", zmco);

    Ok(as5600)
}

// fn create_motor<'d>(
//     timer: impl Peripheral<P = impl LedcTimer> + 'd,
//     channel: impl Peripheral<P = impl LedcChannel> + 'd,
//     pin: impl Peripheral<P = impl OutputPin> + 'd,
// ) -> Result<LedcDriver<'d>> {
//     LedcDriver::new(
//         channel,
//         LedcTimerDriver::new(
//             timer,
//             &TimerConfig::new()
//                 .frequency(50.kHz().into())
//                 .resolution(Resolution::Bits14),
//         )?,
//         pin,
//     )
// }

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Start program");

    let peripherals = Peripherals::take().unwrap();

    // Devices
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = I2cDriver::new(peripherals.i2c0, peripherals.pins.gpio0, peripherals.pins.gpio1, &config)?;
    let mut as5600 = As5600::new(i2c_driver);
    FreeRtos::delay_ms(2000);
    let status = as5600.magnet_status().unwrap();
    let agc = as5600.automatic_gain_control().unwrap();
    let mag = as5600.magnitude().unwrap();
    let zmco = as5600.zmco().unwrap();
    println!("{:?}", status);
    println!("{:?}", agc);
    println!("{:?}", mag);
    println!("{:?}", zmco);

    // let as5600 = create_as5600(
    //     peripherals.i2c0,
    //     peripherals.pins.gpio3,
    //     peripherals.pins.gpio4,
    // )?;

    let timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &TimerConfig::new()
            .frequency(50.Hz())
            .resolution(Resolution::Bits14),
    )?;
    let motor = LedcDriver::new(
        peripherals.ledc.channel0,
        timer_driver,
        peripherals.pins.gpio20,
    )?;
    FreeRtos::delay_ms(5000);

    let mut buttons = Buttons::new(
        peripherals.pins.gpio7,
        peripherals.pins.gpio6,
        peripherals.pins.gpio5,
        peripherals.pins.gpio4,
    );
    buttons.enable_interrupt()?;

    let mut env = env::PendulumEnv::from_devices(as5600, motor);
    let mut sin_policy = sin_policy::SinPolicy::new(1.0);
    let mut evaluator = PendulumEvaluator::new(peripherals.timer00);

    loop {
        polling(&mut env, &mut evaluator, &mut sin_policy);
        buttons.enable_interrupt()?;
    }

    #[allow(unreachable_code)]
    Ok(())
}
