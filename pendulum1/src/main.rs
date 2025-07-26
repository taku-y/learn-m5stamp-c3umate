mod buttons;
mod env;
mod evaluator;
mod manual_policy;
mod sin_policy;

use anyhow::Result;
use as5600::As5600;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use esp_idf_svc::hal::i2c::*;
use esp_idf_svc::hal::ledc::{
    config::TimerConfig, LedcDriver, LedcTimerDriver, Resolution,
};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;

use buttons::Buttons;
use env::PendulumEnv;
use evaluator::PendulumEvaluator;
use sin_policy::SinPolicy;
use manual_policy::ManualPolicy;
use std::sync::atomic::{AtomicU8, Ordering};

static STATE: AtomicU8 = AtomicU8::new(0);

const IDLE: u8 = 0;
const OFFSET_CORRECTION: u8 = 10;
const OFFSET_CORRECTION_END: u8 = 11;
const OFFSET_CORRECTION_CANCEL: u8 = 12;
const POTENTIOMETER_MIN: u8 = 13;
const POTENTIOMETER_MAX: u8 = 14;
const POTENTIOMETER_CANCEL: u8 = 15;
const AUTO_POLICY: u8 = 21;
const MANUAL_POLICY_START: u8 = 22;
const MANUAL_POLICY: u8 = 23;
const TERMINATE: u8 = 255;

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

    // Periherals
    let peripherals = Peripherals::take().unwrap();
    let pin_sda = peripherals.pins.gpio0;
    let pin_scl = peripherals.pins.gpio1;
    let pin_motor = peripherals.pins.gpio20;
    let pin_button1 = peripherals.pins.gpio7;
    let pin_button2 = peripherals.pins.gpio6;
    let pin_button3 = peripherals.pins.gpio5;
    let pin_button4 = peripherals.pins.gpio4;
    let pin_potentiometer = peripherals.pins.gpio3;
    let adc = peripherals.adc1;

    // Devices
    log::info!("Initialize I2C for rotary encoder...");
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = I2cDriver::new(peripherals.i2c0, pin_sda, pin_scl, &config)?;
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

    log::info!("Initialize LEDC for motor control...");
    let timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &TimerConfig::new()
            .frequency(50.Hz())
            .resolution(Resolution::Bits14),
    )?;
    let motor = LedcDriver::new(peripherals.ledc.channel0, timer_driver, pin_motor)?;
    FreeRtos::delay_ms(5000);

    log::info!("Initialize buttons...");
    let mut buttons = Buttons::new(pin_button1, pin_button2, pin_button3, pin_button4);
    buttons.enable_interrupt()?;

    log::info!("Initialize PendulumEnv and SinPolicy...");
    let mut env = PendulumEnv::from_devices(as5600, motor);
    let mut auto_policy = SinPolicy::new(1.0);
    let mut evaluator = PendulumEvaluator::new(peripherals.timer00);

    log::info!("Initialize ManualPolicy...");
    let mut manual_policy = ManualPolicy::new(adc, pin_potentiometer);

    log::info!("Starting main loop");
    loop {
        match get_state() {
            // Idle
            IDLE => {
                log::info!("polling: {}", STATE.load(Ordering::Relaxed));
                FreeRtos::delay_ms(1000);
            }

            // Offset correction
            OFFSET_CORRECTION => {
                // Start pooling loop inside PendulumEnv for offset correction
                env.correct_offset();
                set_state(POTENTIOMETER_MIN);
            }

            POTENTIOMETER_MIN => {
                log::info!("Take minimum potentiometer value");
                FreeRtos::delay_ms(1000);
                let value = manual_policy.take_potentiometer_value(POTENTIOMETER_MIN);

                if get_state() == POTENTIOMETER_CANCEL {
                    set_state(IDLE);
                } else {
                    manual_policy.set_min_limit(value);
                }
            }

            POTENTIOMETER_MAX => {
                log::info!("Take maximum potentiometer value");
                FreeRtos::delay_ms(1000);
                let value = manual_policy.take_potentiometer_value(POTENTIOMETER_MAX);

                if get_state() == POTENTIOMETER_CANCEL {
                    set_state(IDLE);
                } else {
                    manual_policy.set_max_limit(value);
                }
            }

            // Run an episode
            AUTO_POLICY => evaluator.evaluate(&mut auto_policy, &mut env, 0).unwrap(),

            // Run an episode
            MANUAL_POLICY => evaluator.evaluate(&mut manual_policy, &mut env, 0).unwrap(),

            // Terminate the program
            TERMINATE => {
                log::info!("Terminating program...");
                break;
            }

            // Send episode data to the server
            2 => {
                log::info!("polling: {}", STATE.load(Ordering::Relaxed));
                FreeRtos::delay_ms(1000);
                STATE.store(0, Ordering::Relaxed);
            }

            // Receive model parameters from the server
            3 => {
                log::info!("polling: {}", STATE.load(Ordering::Relaxed));
                FreeRtos::delay_ms(1000);
                STATE.store(0, Ordering::Relaxed);
            }

            // Clear the episode data
            4 => {
                log::info!("polling: {}", STATE.load(Ordering::Relaxed));
                FreeRtos::delay_ms(1000);
                STATE.store(0, Ordering::Relaxed);
            }
            _ => {}
        }
        buttons.enable_interrupt()?;
    }

    #[allow(unreachable_code)]
    Ok(())
}

fn get_state() -> u8 {
    STATE.load(Ordering::Relaxed)
}

fn set_state(state: u8) {
    STATE.store(state, Ordering::Relaxed);
}
