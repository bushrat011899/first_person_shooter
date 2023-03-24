use std::time::Duration;

use bevy::{
    prelude::{
        AnimationClip, AnimationPlayer, Bundle, Children, Component, Entity, EventReader,
        EventWriter, Handle, HierarchyQueryExt, Plugin, Query, Res, Scene, With, Without,
    },
    reflect::Reflect,
    time::Time,
};
use bevy_kira_audio::prelude::{Audio, AudioControl, AudioEmitter, AudioSource};

pub struct FirearmPlugin;

impl Plugin for FirearmPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<FirearmEvent<Fire>>()
            .add_event::<FirearmEvent<Fired>>()
            .add_systems((
                process_firearm_fire_requests,
                play_fire_soundeffects,
                play_fire_animation,
            ));
    }
}

pub struct Fire;

pub struct Fired;

pub struct FirearmEvent<EventType> {
    pub details: EventType,
    pub entity: Entity,
}

pub struct FirearmAction {
    pub animation: Handle<AnimationClip>,
    pub sound: Handle<AudioSource>,
    pub cooldown: f32,
}

#[derive(Component, Reflect)]
pub struct FirearmState {
    pub last_fired_seconds: f32,
}

impl Default for FirearmState {
    fn default() -> Self {
        Self {
            last_fired_seconds: f32::NEG_INFINITY
        }
    }
}

#[derive(Component)]
pub struct FirearmActions {
    pub fire: FirearmAction,
}

#[derive(Bundle)]
pub struct FirearmBundle {
    pub model: Handle<Scene>,
    pub actions: FirearmActions,
    pub audio_emitter: AudioEmitter,
    pub state: FirearmState,
}

pub fn process_firearm_fire_requests(
    mut fire_events: EventReader<FirearmEvent<Fire>>,
    mut fired_events: EventWriter<FirearmEvent<Fired>>,
    mut gun_query: Query<(&FirearmActions, &mut FirearmState), With<FirearmActions>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds();

    for fire_event in fire_events.iter() {
        let Ok((actions, mut last_fired)) = gun_query.get_mut(fire_event.entity) else {
            continue;
        };

        // Check if the firearm is on cooldown
        if current_time - last_fired.last_fired_seconds <= actions.fire.cooldown {
            continue;
        }

        last_fired.last_fired_seconds = current_time;

        fired_events.send(FirearmEvent {
            details: Fired,
            entity: fire_event.entity,
        });
    }
}

pub fn play_fire_soundeffects(
    mut fired_events: EventReader<FirearmEvent<Fired>>,
    mut gun_query: Query<(&FirearmActions, &mut AudioEmitter), With<FirearmActions>>,
    audio: Res<Audio>,
) {
    for fired_event in fired_events.iter() {
        let Ok((actions, mut audio_emitter)) = gun_query.get_mut(fired_event.entity) else {
            continue;
        };

        audio_emitter
            .instances
            .push(audio.play(actions.fire.sound.clone_weak()).handle());
    }
}

pub fn play_fire_animation(
    mut fired_events: EventReader<FirearmEvent<Fired>>,
    mut gun_query: Query<&FirearmActions, With<FirearmActions>>,
    children: Query<&Children>,
    mut query: Query<&mut AnimationPlayer, Without<FirearmActions>>,
) {
    for fired_event in fired_events.iter() {
        let Ok(actions) = gun_query.get_mut(fired_event.entity) else {
            continue;
        };

        for child in children.iter_descendants(fired_event.entity) {
            let Ok(mut player) = query.get_mut(child) else {
                continue;
            };

            player
                .set_speed(2.0)
                .play_with_transition(
                    actions.fire.animation.clone_weak(),
                    Duration::from_millis(50),
                )
                .set_elapsed(0.0);

            break;
        }
    }
}
