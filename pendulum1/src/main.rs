mod buttons;
mod env;
mod evaluator;
mod sin_policy;

use anyhow::Result;
use as5600::As5600;
use env::PendulumEnv;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use esp_idf_svc::hal::i2c::*;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
use evaluator::PendulumEvaluator;
use sin_policy::SinPolicy;
use std::sync::atomic::{AtomicU8, Ordering};

use buttons::Buttons;

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

    Ok(As5600::new(i2c_driver))
}

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Start program");

    let peripherals = Peripherals::take().unwrap();

    // Devices
    let as5600 = create_as5600(
        peripherals.i2c0,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
    )?;
    let mut buttons = Buttons::new(
        peripherals.pins.gpio0,
        peripherals.pins.gpio1,
        peripherals.pins.gpio10,
        peripherals.pins.gpio8,
    );
    buttons.enable_interrupt()?;

    let mut env = env::PendulumEnv::from_devices(as5600);
    let mut sin_policy = sin_policy::SinPolicy::new(0.5);
    let mut evaluator = PendulumEvaluator::new(peripherals.timer00);

    loop {
        polling(&mut env, &mut evaluator, &mut sin_policy);
        buttons.enable_interrupt()?;
    }

    #[allow(unreachable_code)]
    Ok(())
}
