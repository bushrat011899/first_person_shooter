use bevy::prelude::*;
use bevy_hanabi::prelude::*;

#[derive(Resource)]
pub struct SparksEffect {
    pub effect: Handle<EffectAsset>,
}

pub fn setup_sparks_particles(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    config: Res<crate::config::Config>,
) {
    let (size, capacity, batch_size) = match config.graphics.particles {
        crate::config::ParticleDetail::Low => (0.3, 1024, 64),
        crate::config::ParticleDetail::Medium => (0.2, 2048, 128),
        crate::config::ParticleDetail::High => (0.1, 4096, 256),
    };

    let batch_size = batch_size as f32;

    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(size));
    size_gradient1.add_key(0.3, Vec2::splat(size));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let effect = effects.add(
        EffectAsset {
            name: "sparks".to_string(),
            capacity,
            spawner: Spawner::once(batch_size.into(), true),
            ..Default::default()
        }
        .init(InitPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 2.,
            dimension: ShapeDimension::Volume,
        })
        .init(InitVelocitySphereModifier {
            center: Vec3::ZERO,
            speed: Value::Uniform((2., 5.)),
        })
        .init(InitLifetimeModifier {
            lifetime: Value::Uniform((0.1, 1.0)),
        })
        .init(InitAgeModifier {
            age: Value::Uniform((0.0, 0.2)),
        })
        .update(LinearDragModifier { drag: 5. })
        .update(AccelModifier::constant(Vec3::new(0., -8., 0.)))
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
        }),
    );

    commands.insert_resource(SparksEffect { effect });
}
