use std::collections::VecDeque;

use bevy::{
    color::palettes::basic::*,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    post_process::bloom::Bloom,
    prelude::*,
};

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct Star;

#[derive(Component)]
struct CameraTarget;

#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct PlanetUi(Entity);

#[derive(Component)]
struct Trail {
    points: VecDeque<Vec2>,
    max_points: usize,
}

#[derive(Resource, Deref, DerefMut)]
struct TrailTimer(Timer);

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

const SIZE_SCALE: f32 = 2.0;
const DIST_SCALE: f32 = 100.0;
const TIME_SCALE: f32 = 1.0;
const G: f32 = 8e-7;
const TRAIL_MAX_LENGTH: usize = 50;
const TRAIL_INTERVAL: f32 = 0.1;
const SOLAR_MASS: f32 = 330000.0;
const CAMERA_TARGET_SPEED: f32 = 200.0;
const CAMERA_DECAY_RATE: f32 = 2.2;

struct PlanetStats {
    name: &'static str,
    mass: f32,
    size: f32,
    initial_radius: f32,
}

const PLANETS: [PlanetStats; 9] = [
    PlanetStats {
        name: "Mercury",
        mass: 0.055,
        size: 0.38,
        initial_radius: 0.39,
    },
    PlanetStats {
        name: "Venus",
        mass: 0.815,
        size: 0.95,
        initial_radius: 0.72,
    },
    PlanetStats {
        name: "Earth",
        mass: 1.0,
        size: 1.0,
        initial_radius: 1.0,
    },
    PlanetStats {
        name: "Mars",
        mass: 0.107,
        size: 0.53,
        initial_radius: 1.52,
    },
    PlanetStats {
        name: "Jupiter",
        mass: 317.8,
        size: 11.21,
        initial_radius: 5.2,
    },
    PlanetStats {
        name: "Saturn",
        mass: 95.2,
        size: 9.45,
        initial_radius: 9.58,
    },
    PlanetStats {
        name: "Uranus",
        mass: 14.5,
        size: 4.01,
        initial_radius: 19.22,
    },
    PlanetStats {
        name: "Neptune",
        mass: 17.1,
        size: 3.88,
        initial_radius: 30.05,
    },
    PlanetStats {
        name: "Pluto",
        mass: 0.002,
        size: 0.18,
        initial_radius: 38.48,
    },
];

fn setup_planets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut colors: ResMut<Assets<ColorMaterial>>,
) {
    let mesh_color = colors.add(Color::from(WHITE));

    for planet in PLANETS {
        commands.spawn((
            Planet,
            Name::from(planet.name),
            Mass(planet.mass),
            Mesh2d(meshes.add(Circle::new((1.0 + planet.size.log10()) * SIZE_SCALE))),
            MeshMaterial2d(mesh_color.clone()),
            Position(Vec2::new(planet.initial_radius, 0.0)),
            Transform::default(),
            Velocity(Vec2::new(
                0.0,
                (G * SOLAR_MASS).sqrt() / planet.initial_radius.sqrt(),
            )),
            Trail {
                points: VecDeque::new(),
                max_points: (TRAIL_MAX_LENGTH as f32 * planet.initial_radius) as usize,
            },
        ));

        info!("{} size is {}", planet.name, 1.0 + planet.size.log10())
    }
    commands.spawn((
        Star,
        Mass(SOLAR_MASS),
        Mesh2d(meshes.add(Circle::new(0.00465 * DIST_SCALE))),
        MeshMaterial2d(colors.add(Color::srgb(10., 7., 1.))),
        Transform::default(),
    ));

    commands.spawn((CameraTarget, Transform::default()));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Bloom::NATURAL));
}

fn update_position(planets: Query<(&Velocity, &mut Position), With<Planet>>, time: Res<Time>) {
    let dt = time.delta_secs() / TIME_SCALE;

    for (velocity, mut position) in planets {
        position.0 += velocity.0 * dt;
    }
}

fn update_velocity(
    planets: Query<(&mut Velocity, &Position), With<Planet>>,
    star_mass: Single<&Mass, With<Star>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs() / TIME_SCALE;

    for (mut velocity, position) in planets {
        let r2 = position.0.length_squared();

        if r2 == 0.0 {
            continue;
        }

        let accel = -position.0.normalize_or_zero() * G * star_mass.0 / r2;
        velocity.0 += accel * dt;
    }
}

fn draw_planets(planets: Query<(&Position, &mut Transform), With<Planet>>) {
    for (position, mut transform) in planets {
        transform.translation = position.0.extend(0.0) * DIST_SCALE;
    }
}

fn update_trails(
    trails: Query<(&Transform, &mut Trail)>,
    time: Res<Time>,
    mut timer: ResMut<TrailTimer>,
) {
    timer.tick(time.delta());

    if timer.just_finished() {
        for (transform, mut trail) in trails {
            let pos = transform.translation.truncate();

            trail.points.push_back(pos);

            if trail.points.len() > trail.max_points {
                trail.points.pop_front();
            }
        }
    }
}

fn draw_trails(
    trails: Query<&Trail>,
    camera: Single<&Projection, With<Camera>>,
    mut gizmos: Gizmos,
) {
    let Projection::Orthographic(camera_projection) = *camera else {
        return;
    };

    for trail in trails {
        for point in &trail.points {
            gizmos.circle_2d(
                Isometry2d::from_translation(*point),
                1.0 * camera_projection.scale,
                WHITE,
            );
        }
    }
}

fn setup_ui(mut commands: Commands, planet_ids: Query<Entity, With<Planet>>) {
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: px(20),
            right: px(20),
            row_gap: px(4),
            flex_direction: FlexDirection::Column,
            ..default()
        },))
        .with_children(|parent| {
            for entity in &planet_ids {
                parent.spawn((
                    Text::new(""),
                    TextColor(Color::WHITE),
                    TextLayout::justify(Justify::Left),
                    PlanetUi(entity),
                ));
            }
        });
}

fn update_ui(
    planets: Query<(&Position, &Name), With<Planet>>,
    ui_text: Query<(&mut Text, &PlanetUi)>,
) {
    for (mut text, planet_id) in ui_text {
        let Ok((position, name)) = planets.get(planet_id.0) else {
            continue;
        };

        text.0 = format!("{} orbit radius: {:.2} AU", name, position.0.length())
    }
}

fn move_camera_target(
    mut camera_target: Single<&mut Transform, With<CameraTarget>>,
    time: Res<Time<Real>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let mut direction = Vec2::ZERO;

    if input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }

    if input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }

    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }

    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    camera_target.translation +=
        (direction.normalize_or_zero() * time.delta_secs() * CAMERA_TARGET_SPEED).extend(0.0);
}

fn update_camera(
    camera: Single<(&mut Transform, &mut Projection), (With<Camera2d>, Without<CameraTarget>)>,
    mut camera_target: Single<&mut Transform, (With<CameraTarget>, Without<Camera2d>)>,
    time: Res<Time<Real>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let (mut camera_transform, mut projection) = camera.into_inner();

    let Projection::Orthographic(camera_projection) = &mut *projection else {
        return;
    };

    let direction = Vec3::new(
        camera_target.translation.x,
        camera_target.translation.y,
        camera_transform.translation.z,
    );

    camera_transform
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());

    if mouse_button.pressed(MouseButton::Left) {
        let pan_delta = mouse_motion.delta * camera_projection.scale;

        camera_transform.translation.x -= pan_delta.x;
        camera_transform.translation.y += pan_delta.y;

        camera_target.translation.x -= pan_delta.x;
        camera_target.translation.y += pan_delta.y
    }

    camera_projection.scale *= 0.9f32.powf(scroll.delta.y);

    if input.just_pressed(KeyCode::Minus) {
        camera_projection.scale /= 0.8f32;
    }

    if input.just_pressed(KeyCode::Equal) {
        camera_projection.scale *= 0.8f32;
    }
}

fn pause_game(mut time: ResMut<Time<Virtual>>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(TrailTimer(Timer::from_seconds(
            TRAIL_INTERVAL,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, (setup_camera, (setup_planets, setup_ui).chain()))
        .add_systems(FixedUpdate, (update_velocity, update_position).chain())
        .add_systems(
            Update,
            (
                pause_game,
                update_ui,
                (move_camera_target, update_camera).chain(),
                (draw_planets, update_trails, draw_trails).chain(),
            ),
        )
        .run();
}
