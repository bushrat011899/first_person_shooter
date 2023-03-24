use bevy::prelude::*;
use bevy_hanabi::prelude::*;

#[derive(Resource)]
pub struct SmokeCloudEffect {
    pub effect: Handle<EffectAsset>,
}

pub fn setup_smoke_particles(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    config: Res<crate::config::Config>,
) {
    let (transparency, capacity, batch_size) = match config.graphics.particles {
        crate::config::ParticleDetail::Low => (0.5, 4096, 256),
        crate::config::ParticleDetail::Medium => (0.2, 16384, 1024),
        crate::config::ParticleDetail::High => (0.1, 32768, 4096),
    };

    let batch_size = batch_size as f32;

    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(0.0, 0.0, 0.0, transparency));
    color_gradient1.add_key(1.0, Vec4::new(0.3, 0.3, 0.3, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.1));
    size_gradient1.add_key(1.0, Vec2::splat(1.0));

    let effect = effects.add(
        EffectAsset {
            name: "smoke".to_string(),
            capacity,
            spawner: Spawner::new(batch_size.into(), 0.1.into(), f32::INFINITY.into()),
            ..Default::default()
        }
        .init(InitPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 2.,
            dimension: ShapeDimension::Volume,
        })
        .init(InitVelocitySphereModifier {
            center: Vec3::ZERO,
            speed: Value::Uniform((2., 10.)),
        })
        .init(InitLifetimeModifier {
            lifetime: Value::Uniform((0.1, 4.0)),
        })
        .init(InitAgeModifier {
            age: Value::Uniform((0.0, 0.2)),
        })
        .update(LinearDragModifier { drag: 5. })
        .update(AccelModifier::constant(Vec3::new(0., -1., 0.)))
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
        })
        .render(BillboardModifier),
    );

    commands.insert_resource(SmokeCloudEffect { effect });
}
