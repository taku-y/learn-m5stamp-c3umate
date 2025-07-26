use crate::env::{PendulumEnv, PendulumEnvAct, PendulumEnvObs};
use border_core::Policy;

/// A simple policy that uses a sine function to control the pendulum.
pub struct SinPolicy {
    frequency: f32,
    time: f32,
}

impl SinPolicy {
    pub fn new(frequency: f32) -> Self {
        SinPolicy {
            frequency,
            time: 0.0, // Initialize time to zero
        }
    }
}

impl<'d> Policy<PendulumEnv<'d>> for SinPolicy {
    fn sample(&mut self, _obs: &PendulumEnvObs) -> PendulumEnvAct {
        self.time += 0.025; // Increment time
        (self.frequency * self.time).sin().into()
    }
}
