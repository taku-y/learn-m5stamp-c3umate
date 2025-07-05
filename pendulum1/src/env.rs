use anyhow::Result;
use as5600::As5600;
use border_core::{record::Record, Act, Env, Obs, Step};
use esp_idf_svc::hal::i2c::I2cDriver;

pub struct PendulumEnv<'d> {
    sensor: As5600<I2cDriver<'d>>,
}

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

impl From<f32> for PendulumEnvAct {
    fn from(action: f32) -> Self {
        PendulumEnvAct { action }
    }
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
        let value = self.sensor.angle().unwrap();
        let obs = PendulumEnvObs { value: value as _ };
        let act = action.clone();
        println!("{:?}", value);

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
        todo!();
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
    pub fn from_devices(sensor: As5600<I2cDriver<'d>>) -> Self {
        PendulumEnv { sensor }
    }
}
