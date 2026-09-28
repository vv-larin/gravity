use bevy::{color::palettes::basic::*, prelude::*};

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct Star;

#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct Trail {
    points: Vec<Vec2>,
    max_points: usize,
    mesh: Option<Handle<Mesh>>,
}

#[derive(Resource, Deref, DerefMut)]
struct TrailTimer(Timer);

#[derive(Component)]
struct Velocity(Vec2);

const SIZE_SCALE: f32 = 10.0;
const DIST_SCALE: f32 = 300.0;
const TIME_SCALE: f32 = 5.0;
const SQRT_GM: f32 = 0.5138;
const G: f32 = 8e-7;
const TRAIL_MAX_LENGTH: usize = 100;
const TRAIL_INTERVAL: f32 = 0.5;

struct PlanetStats {
    name: &'static str,
    mass: f32,
    size: f32,
    initial_radius: f32,
}

const PLANETS: [PlanetStats; 4] = [
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
    // PlanetStats {
    //     name: "Jupiter",
    //     mass: 317.8,
    //     size: 11.21,
    //     initial_radius: 5.2,
    // },
    // PlanetStats {
    //     name: "Saturn",
    //     mass: 95.2,
    //     size: 9.45,
    //     initial_radius: 9.58,
    // },
    // PlanetStats {
    //     name: "Uranus",
    //     mass: 14.5,
    //     size: 4.01,
    //     initial_radius: 19.22,
    // },
    // PlanetStats {
    //     name: "Neptune",
    //     mass: 17.1,
    //     size: 3.88,
    //     initial_radius: 30.05,
    // },
    // PlanetStats {
    //     name: "Pluto",
    //     mass: 0.002,
    //     size: 0.18,
    //     initial_radius: 38.48,
    // },
];

fn setup_planets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut colors: ResMut<Assets<ColorMaterial>>,
) {
    for planet in PLANETS {
        commands.spawn((
            Planet,
            Name::from(planet.name),
            Mass(planet.mass),
            Mesh2d(meshes.add(Circle::new((1.0 + planet.size.log10()) * SIZE_SCALE))),
            MeshMaterial2d(colors.add(Color::from(WHITE))),
            Transform::from_xyz(planet.initial_radius * DIST_SCALE, 0.0, 0.0),
            Velocity(Vec2::new(
                0.0,
                (SQRT_GM / planet.initial_radius.sqrt()) * DIST_SCALE / TIME_SCALE,
            )),
            Trail {
                points: Vec::new(),
                max_points: TRAIL_MAX_LENGTH,
                mesh: None,
            },
        ));
        println!("{} size is {}", planet.name, 1.0 + planet.size.log10())
    }
    commands.spawn((
        Star,
        Mass(330000.0),
        // Mesh2d(meshes.add(Circle::new(planet.size * SIZE_SCALE))),
        Mesh2d(meshes.add(Circle::new(25.0))),
        MeshMaterial2d(colors.add(Color::from(YELLOW))),
        Transform::default(),
    ));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn update_position(planets: Query<(&Velocity, &mut Transform), With<Planet>>, time: Res<Time>) {
    let dt = time.delta_secs();

    for (velocity, mut transform) in planets {
        transform.translation += velocity.0.extend(0.0) * dt;
    }
}

fn update_velocity(
    planets: Query<(&mut Velocity, &Transform), With<Planet>>,
    star_mass: Single<&Mass, With<Star>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut velocity, transform) in planets {
        let r2 = (transform.translation / DIST_SCALE).length_squared();
        let accel =
            -1.0 * transform.translation.truncate().normalize_or_zero() * G * star_mass.0 / r2;
        velocity.0 += accel * dt * DIST_SCALE / TIME_SCALE.powi(2);
    }
}

fn update_trails(
    trails: Query<(&GlobalTransform, &mut Trail)>,
    time: Res<Time>,
    mut timer: ResMut<TrailTimer>,
) {
    timer.tick(time.delta());

    if timer.just_finished() {
        for (transform, mut trail) in trails {
            let pos = transform.translation().truncate();

            trail.points.push(pos);

            if trail.points.len() > trail.max_points {
                trail.points.remove(0);
            }

            if trail.points.len() < 2 {
                continue;
            }
        }
    }
}

fn draw_trails(trails: Query<&Trail>, mut gizmos: Gizmos) {
    for trail in trails {
        for point in &trail.points {
            gizmos.circle_2d(Isometry2d::from_translation(*point), 2.0, WHITE);
        }
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(20.0),
            right: px(20.0),
            ..default()
        },
        Text::new("Some text"),
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Left),
    ));
}

fn update_ui(planets: Query<(&Transform, &Name), With<Planet>>, mut ui_text: Query<&mut Text>) {
    for (transform, name) in planets {
        if name.as_str() == "Earth" {
            let mut text = ui_text.single_mut().expect("More than one UI element");
            text.0 = transform.translation.distance(Vec3::ZERO).to_string();
        } else {
            continue;
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
        .add_systems(Startup, (setup_camera, setup_planets, setup_ui))
        .add_systems(Update, update_ui)
        .add_systems(
            Update,
            (update_velocity, update_position, update_trails, draw_trails).chain(),
        )
        .run();
}
