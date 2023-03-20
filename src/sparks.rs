use bevy::prelude::*;
use bevy_hanabi::prelude::*;

#[derive(Resource)]
pub struct BulletImpactEffect {
    pub effect: Handle<EffectAsset>,
}

#[derive(Resource)]
pub struct SmokeCloudEffect {
    pub effect: Handle<EffectAsset>,
}

pub fn setup_sparks_particles(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.02));
    size_gradient1.add_key(0.3, Vec2::splat(0.02));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let effect = effects.add(
        EffectAsset {
            name: "firework".to_string(),
            capacity: 32768,
            spawner: Spawner::once(200.0.into(), true),
            ..Default::default()
        }
        .init(InitPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 2.,
            dimension: ShapeDimension::Volume,
        })
        .init(InitVelocitySphereModifier {
            center: Vec3::ZERO,
            // Give a bit of variation by randomizing the initial speed
            speed: Value::Uniform((2., 5.)),
        })
        .init(InitLifetimeModifier {
            // Give a bit of variation by randomizing the lifetime per particle
            lifetime: Value::Uniform((0.1, 1.0)),
        })
        .init(InitAgeModifier {
            // Give a bit of variation by randomizing the age per particle. This will control the
            // starting color and starting size of particles.
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

    commands.insert_resource(BulletImpactEffect { effect });
}

pub fn setup_smoke_particles(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(0.0, 0.0, 0.0, 0.1));
    color_gradient1.add_key(1.0, Vec4::new(0.3, 0.3, 0.3, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.1));
    size_gradient1.add_key(1.0, Vec2::splat(1.0));

    let effect = effects.add(
        EffectAsset {
            name: "smoke".to_string(),
            capacity: 32768,
            spawner: Spawner::once(3000.0.into(), true),
            ..Default::default()
        }
        .init(InitPositionSphereModifier {
            center: Vec3::ZERO,
            radius: 2.,
            dimension: ShapeDimension::Volume,
        })
        .init(InitVelocitySphereModifier {
            center: Vec3::ZERO,
            // Give a bit of variation by randomizing the initial speed
            speed: Value::Uniform((2., 10.)),
        })
        .init(InitLifetimeModifier {
            // Give a bit of variation by randomizing the lifetime per particle
            lifetime: Value::Uniform((0.1, 4.0)),
        })
        .init(InitAgeModifier {
            // Give a bit of variation by randomizing the age per particle. This will control the
            // starting color and starting size of particles.
            age: Value::Uniform((0.0, 0.2)),
        })
        .update(LinearDragModifier { drag: 5. })
        .update(AccelModifier::constant(Vec3::new(0., -1., 0.)))
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
        }),
    );

    commands.insert_resource(SmokeCloudEffect { effect });
}
