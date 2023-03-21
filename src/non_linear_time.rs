use bevy::prelude::*;

#[derive(Resource, Default, Reflect, Hash)]
#[reflect(Resource, Hash)]
pub struct ExactTime {
    pub tick_rate: u16,
    pub tick: u16,
    pub seconds: u32,
}

impl ExactTime {
    pub fn tick(&mut self) {
        self.tick += 1;

        while self.tick >= self.tick_rate {
            self.tick -= self.tick_rate;
            self.seconds += 1;
        }
    }
}

pub fn track_exact_time(mut time: ResMut<ExactTime>) {
    time.tick();
}
