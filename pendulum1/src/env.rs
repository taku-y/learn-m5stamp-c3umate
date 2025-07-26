use anyhow::Result;
use as5600::As5600;
use border_core::{record::Record, Act, Env, Obs, Step};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::{i2c::I2cDriver, ledc::LedcDriver};
use std::sync::atomic::Ordering;

#[derive(Debug, Clone)]
pub struct PendulumEnvObs {
    #[allow(dead_code)]
    value: f32,
}

impl Obs for PendulumEnvObs {
    fn len(&self) -> usize {
        1
    }
}

impl PendulumEnvObs {
    /// Get the observation value.
    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct PendulumEnvAct {
    #[allow(dead_code)]
    action: f32,
}

impl Act for PendulumEnvAct {
    fn len(&self) -> usize {
        1
    }
}

impl PendulumEnvAct {
    /// Get the action value.
    pub fn value(&self) -> f32 {
        self.action
    }
}

impl From<f32> for PendulumEnvAct {
    fn from(action: f32) -> Self {
        PendulumEnvAct { action }
    }
}

pub struct PendulumEnv<'d> {
    sensor: As5600<I2cDriver<'d>>,
    motor: LedcDriver<'d>,
    min_limit: u32,
    max_limit: u32,
    offset: f32,
    direction: f32,
    scale: f32,
}

impl<'d> Env for PendulumEnv<'d> {
    type Config = ();
    type Act = PendulumEnvAct;
    type Obs = PendulumEnvObs;
    type Info = ();

    /// Use from_devices() to create a new PendulumEnv.
    fn build(_config: &Self::Config, _seed: i64) -> Result<Self> {
        unimplemented!();
    }

    fn step(&mut self, action: &PendulumEnvAct) -> (Step<Self>, Record) {
        let value = self.angle();
        let obs = PendulumEnvObs { value: value as _ };
        let act = action.clone();

        // Take action
        let value = 180.0 * (self.scale * act.value() + 1.0) * 0.5;
        let duty = self.map(value as _);
        self.motor.set_duty(duty).unwrap();

        println!(
            "obs, act, duty = ({:?}, {:?}, {:?})",
            obs.value(),
            act.value(),
            duty
        );

        let step = Step::new(
            obs,
            act,
            vec![0.0], // Placeholder for reward
            vec![0],
            vec![0],
            (),
            None,
        );

        (step, Record::empty())
    }

    fn reset(&mut self, _is_done: Option<&Vec<i8>>) -> anyhow::Result<Self::Obs> {
        // let _ = self.motor.set_duty(self.map(90)).unwrap();
        println!("Resetting pendulum to 90 degrees (duty={})", self.map(90));
        FreeRtos::delay_ms(2000); // Allow time for the motor to move
        let value = self.angle();
        Ok(PendulumEnvObs { value: value as _ })
    }

    fn reset_with_index(&mut self, _ix: usize) -> anyhow::Result<Self::Obs> {
        todo!();
    }

    fn step_with_reset(&mut self, _a: &Self::Act) -> (Step<Self>, Record) {
        todo!();
    }
}

impl<'d> PendulumEnv<'d> {
    /// Create a new PendulumEnv from devices on ESP32.
    pub fn from_devices(sensor: As5600<I2cDriver<'d>>, motor: LedcDriver<'d>) -> Self {
        let max_duty = motor.get_max_duty();
        let min_limit = max_duty * 5 / 10 / 20;
        let max_limit = max_duty * 24 / 10 / 20;
        println!("Min Limit {}", min_limit);
        println!("Max Limit {}", max_limit);
        FreeRtos::delay_ms(2000);
        PendulumEnv {
            sensor,
            motor,
            min_limit,
            max_limit,
            offset: 0.0,
            direction: 0.0,
            scale: 0.6,
        }
    }

    // Function that maps one range to another
    fn map(&self, x: u32) -> u32 {
        let in_min = 0;
        let in_max = 180;
        let out_min = self.min_limit;
        let out_max = self.max_limit;

        (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
    }

    /// Return the current angle of the pendulum in radians.
    fn angle(&mut self) -> f32 {
        // Get the angle in radians
        let angle = self.sensor.angle().unwrap_or(0) as f32 * std::f32::consts::PI / 2048.0;
        let angle = self.direction * angle - self.offset;
        if angle < -std::f32::consts::PI {
            angle + 2.0 * std::f32::consts::PI
        } else if angle > std::f32::consts::PI {
            angle - 2.0 * std::f32::consts::PI
        } else {
            angle
        }
    }

    pub fn correct_offset(&mut self) {
        let offset = self.sensor.angle().unwrap();
        log::info!("Offset: {}", offset);
        log::info!("Starting offset correction in 1 second...");
        FreeRtos::delay_ms(1000);

        loop {
            let angle = self.sensor.angle().unwrap();
            log::info!("Current angle: {} ({})", angle, self.angle());

            if crate::STATE.load(Ordering::Relaxed) == crate::OFFSET_CORRECTION_END {
                self.offset = offset as f32 * std::f32::consts::PI / 2048.0;
                self.direction = get_direction(angle, offset) as f32;
                if self.direction == 1.0 {
                    log::info!("Counter-clockwise direction is positive.");
                } else {
                    log::info!("Counter-clockwise direction is negative.");
                }
                log::info!("Offset correction completed.");
                FreeRtos::delay_ms(1000);
                break;
            } else if crate::STATE.load(Ordering::Relaxed) == crate::OFFSET_CORRECTION_CANCEL {
                log::info!("Offset correction cancelled.");
                FreeRtos::delay_ms(1000);
                break;
            }
            FreeRtos::delay_ms(100);
        }
    }
}

/// Check the direction of the rotary encoder.
///
/// This function should be called when the pendulum is physically rotated counter-clockwise
/// (roughly 10-20 degrees). If the angle is larger than the offset, it returns 1, meaning that
/// the counter-clockwise direction corresponds to a positive angle. Otherwise, it returns -1,
/// meaning that the counter-clockwise direction corresponds to a negative angle.
///
/// This function handles the case where the angle exceeds 4096, which is the maximum value of
/// the encoder with a 12-bit resolution.
fn get_direction(angle: u16, offset: u16) -> i8 {
    if angle > offset {
        if angle - offset > 2048 {
            -1 // Counter-clockwise is negative
        } else {
            1 // Counter-clockwise is positive
        }
    } else {
        if offset - angle > 2048 {
            1 // Counter-clockwise is positive
        } else {
            -1 // Counter-clockwise is negative
        }
    }
}
