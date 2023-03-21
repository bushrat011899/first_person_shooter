use bevy::prelude::*;

use crate::{firearm::{FirearmEvent, Fired}, player};

pub fn increase_fog_after_shots(
    mut fired_events: EventReader<FirearmEvent<Fired>>,
    mut query: Query<&mut FogSettings, With<player::Head>>,
) {
    let fog_steps: u16 = fired_events.iter().count().try_into().unwrap_or(u16::MAX);

    if fog_steps == 0 {
        return;
    };

    for mut settings in query.iter_mut() {
        let density = match settings.falloff {
            FogFalloff::Exponential { density } => density,
            _ => 0.1,
        };

        let density = density * 1.2_f32.powf(fog_steps.into());

        settings.falloff = FogFalloff::Exponential { density };
    }
}

pub fn clear_fog_over_time(time: Res<Time>, mut query: Query<&mut FogSettings, With<player::Head>>) {
    let dt = time.delta_seconds();

    for mut settings in query.iter_mut() {
        let density = match settings.falloff {
            FogFalloff::Exponential { density } => density,
            _ => 0.1,
        };

        let density = if density < 0.1 {
            0.1
        } else {
            density - 0.01 * dt
        };

        settings.falloff = FogFalloff::Exponential { density };
    }
}