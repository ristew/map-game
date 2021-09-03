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

pub fn individual_event(probability: f64) -> bool {
    random::<f64>() < probability
}

pub fn logistic(x: f64) -> f64 {
    0.5 + 0.5 * (x / 2.0).tanh()
}

pub fn dev_mean_sample(stddev: f64, mean: f64) -> f64 {
    thread_rng().sample::<f64, StandardNormal>(StandardNormal) * stddev + mean
}

pub fn positive_isample(stddev: usize, mean: usize) -> usize {
    dev_mean_sample(stddev as f64, mean as f64).max(0.0).round() as usize
}

pub fn sample(stddev: f64) -> f64 {
    dev_mean_sample(stddev, 0.0)
}
