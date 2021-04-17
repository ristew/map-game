use bevy::prelude::*;
use rand::prelude::*;

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
