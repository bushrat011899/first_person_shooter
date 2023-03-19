use std::time::Duration;

use bevy::{prelude::{HierarchyQueryExt, AnimationPlayer, Children, Res, Without, With, Query, EventReader, Bundle, Component, Handle, AnimationClip, Scene, Entity, EventWriter, Commands}, time::Time};
use bevy_kira_audio::prelude::{Audio, AudioControl, AudioEmitter, AudioSource};

pub struct Fire;

pub struct Fired;

pub struct FirearmEvent<EventType> {
    pub details: EventType,
    pub entity: Entity
}

pub struct FirearmAction {
    pub animation: Handle<AnimationClip>,
    pub sound: Handle<AudioSource>,
    pub cooldown: f32,
}

#[derive(Component)]
pub struct FirearmLastFired {
    pub elapsed_time_seconds: f32
}

#[derive(Component)]
pub struct FirearmActions {
    pub fire: FirearmAction
}

#[derive(Bundle)]
pub struct FirearmBundle {
    pub model: Handle<Scene>,
    pub actions: FirearmActions,
    pub audio_emitter: AudioEmitter
}

pub fn fire_firearms(
    mut commands: Commands,
    mut fire_events: EventReader<FirearmEvent<Fire>>,
    mut fired_events: EventWriter<FirearmEvent<Fired>>,
    mut gun_query: Query<(&FirearmActions, &mut AudioEmitter, Option<&mut FirearmLastFired>), With<FirearmActions>>,
    children: Query<&Children>,
    mut query: Query<&mut AnimationPlayer, Without<FirearmActions>>,
    audio: Res<Audio>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds();

    for fire_event in fire_events.iter() {
        let Ok((actions, mut audio_emitter, last_fired)) = gun_query.get_mut(fire_event.entity) else {
            continue;
        };

        // Check if the firearm is on cooldown
        match last_fired {
            Some(mut last_fired) => {
                if current_time - last_fired.elapsed_time_seconds <= actions.fire.cooldown {
                    continue;
                }

                last_fired.elapsed_time_seconds = current_time;
            },
            None => {
                commands.entity(fire_event.entity).insert(FirearmLastFired { elapsed_time_seconds: current_time });
            },
        };

        audio_emitter
            .instances
            .push(audio.play(actions.fire.sound.clone_weak()).handle());

        for child in children.iter_descendants(fire_event.entity) {
            let Ok(mut player) = query.get_mut(child) else {
                continue;
            };

            player
                .set_speed(2.0)
                .play_with_transition(
                    actions.fire.animation.clone_weak(),
                    Duration::from_millis(10),
                )
                .set_elapsed(0.0);
            
            break;
        }

        fired_events.send(FirearmEvent { details: Fired, entity: fire_event.entity });
    }
}
