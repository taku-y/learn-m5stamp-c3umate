use anyhow::Result;
use as5600::As5600;
use border_core::{record::Record, Act, Env, Obs, Step};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::{i2c::I2cDriver, ledc::LedcDriver};

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
        let value = 180.0 * (2.0 * act.value() + 1.0);
        let duty = self.map(value as _);
        // self.motor.set_duty(duty).unwrap();
        println!("obs, act, duty = ({:?}, {:?}, {:?})", obs.value(), act.value(), duty);

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
        println!("Max Duty {}", max_limit);
        FreeRtos::delay_ms(2000);
        PendulumEnv {
            sensor,
            motor,
            min_limit,
            max_limit,
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

    fn angle(&mut self) -> f32 {
        self.sensor.angle().unwrap() as _
    }
}
