use rand::prelude::*;
use rand_distr::StandardNormal;

pub trait EventSpawner {
    type Event;

    fn spawn(&self) -> Self::Event;
}
pub struct RandomEventGenerator<T> where T: EventSpawner {
    spawner: T,
    probability: f32,
}

impl <T> RandomEventGenerator<T> where T: EventSpawner {
    pub fn try_spawn(&self) -> Option<T::Event> {
        if random::<f32>() < self.probability {
            Some(self.spawner.spawn())
        } else {
            None
        }
    }
}

pub fn dev_mean_sample(stddev: f64, mean: f64) -> f64 {
    thread_rng().sample::<f64, StandardNormal>(StandardNormal) * stddev + mean
}
