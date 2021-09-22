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

pub fn individual_event(probability: f32) -> bool {
    random::<f32>() < probability
}

pub fn logistic(x: f32) -> f32 {
    0.5 + 0.5 * (x / 2.0).tanh()
}

pub fn dev_mean_sample(stddev: f32, mean: f32) -> f32 {
    thread_rng().sample::<f32, StandardNormal>(StandardNormal) * stddev + mean
}

pub fn positive_isample(stddev: isize, mean: isize) -> isize {
    dev_mean_sample(stddev as f32, mean as f32).max(0.0).round() as isize
}

pub fn sample(stddev: f32) -> f32 {
    dev_mean_sample(stddev, 0.0)
}
