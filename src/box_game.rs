use bevy::prelude::*;
use bevy_ggrs::{PlayerInputs, Rollback, RollbackIdProvider, Session};
use bevy_matchbox::prelude::*;
use bytemuck::{Pod, Zeroable};
use ggrs::{Config, PlayerHandle};
use std::{hash::Hash, ops::Mul};

const BLUE: Color = Color::rgb(0.8, 0.6, 0.2);
const ORANGE: Color = Color::rgb(0., 0.35, 0.8);
const MAGENTA: Color = Color::rgb(0.9, 0.2, 0.2);
const GREEN: Color = Color::rgb(0.35, 0.7, 0.35);
const PLAYER_COLORS: [Color; 4] = [BLUE, ORANGE, MAGENTA, GREEN];
const PLAYER_NAMES: [&str; 4] = ["BLUE", "ORANGE", "MAGENTA", "GREEN"];

const INPUT_UP: u8 = 1 << 0;
const INPUT_DOWN: u8 = 1 << 1;
const INPUT_LEFT: u8 = 1 << 2;
const INPUT_RIGHT: u8 = 1 << 3;

const MOVEMENT_SPEED: f32 = 0.005;
const MAX_SPEED: f32 = 0.05;
const FRICTION: f32 = 0.9;
const PLANE_SIZE: f32 = 5.0;
const CUBE_SIZE: f32 = 0.2;

const ENEMY_RADIUS: f32 = 0.5;

/// You need to define a config struct to bundle all the generics of GGRS. You can safely ignore
/// `State` and leave it as u8 for all GGRS functionality.
/// TODO: Find a way to hide the state type.
#[derive(Debug)]
pub struct GGRSConfig;
impl Config for GGRSConfig {
    type Input = BoxInput;
    type State = u8;
    type Address = PeerId;
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
pub struct BoxInput {
    pub inp: u8,
}

#[derive(Default, Component)]
pub struct Player {
    pub handle: usize,
}

#[derive(Default, Reflect, Component)]
pub struct Score {
    pub highscore: u32,
    pub current: u32,
    pub last_death_frame: u32,
}

// Marker component for Enemy
#[derive(Component)]
pub struct Enemy;

// Marker components for UI
#[derive(Component)]
pub struct ScoreText;

// Components that should be saved/loaded need to implement the `Reflect` trait
#[derive(Default, Reflect, Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// You can also register resources.
#[derive(Resource, Default, Reflect, Hash)]
#[reflect(Resource, Hash)]
pub struct FrameCount {
    pub frame: u32,
}

pub fn input(_handle: In<PlayerHandle>, keyboard_input: Res<Input<KeyCode>>) -> BoxInput {
    let mut input: u8 = 0;

    if keyboard_input.pressed(KeyCode::W) {
        input |= INPUT_UP;
    }
    if keyboard_input.pressed(KeyCode::A) {
        input |= INPUT_LEFT;
    }
    if keyboard_input.pressed(KeyCode::S) {
        input |= INPUT_DOWN;
    }
    if keyboard_input.pressed(KeyCode::D) {
        input |= INPUT_RIGHT;
    }

    BoxInput { inp: input }
}

pub fn setup_scene_system(
    mut commands: Commands,
    mut rip: ResMut<RollbackIdProvider>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    session: Res<Session<GGRSConfig>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    asset_server: Res<AssetServer>,
) {
    let num_players = match &*session {
        Session::SyncTestSession(s) => s.num_players(),
        Session::P2PSession(s) => s.num_players(),
        Session::SpectatorSession(s) => s.num_players(),
    };

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: PLANE_SIZE,
            ..default()
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    //enemy
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: ENEMY_RADIUS,
                sectors: 12,
                stacks: 3,
            })),
            material: materials.add(Color::RED.into()),
            transform: Transform {
                translation: Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                ..default()
            },
            ..default()
        },
        Velocity::default(),
        Enemy,
        Rollback::new(rip.next_id()),
    ));

    //score ui
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(15.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Start,
                ..default()
            },
            background_color: Color::rgb(0.43, 0.41, 0.38).into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    style: Style {
                        align_self: AlignSelf::Start,
                        justify_content: JustifyContent::Start,
                        ..default()
                    },
                    text: Text::from_section(
                        "Scores",
                        TextStyle {
                            font: asset_server.load("fonts/quicksand-light.ttf"),
                            font_size: 24.,
                            color: Color::BLACK,
                        },
                    ),
                    ..default()
                })
                .insert(ScoreText);
        });

    // player cube - just spawn whatever entity you want, then add a `Rollback` component with a
    // unique id (for example through the `RollbackIdProvider` resource). Every entity that you
    // want to be saved/loaded needs a `Rollback` component with a unique rollback id.
    // When loading entities from the past, this extra id is necessary to connect entities over
    // different game states
    let r = PLANE_SIZE / 4.;

    for handle in 0..num_players {
        let rot = handle as f32 / num_players as f32 * 2. * std::f32::consts::PI;
        let x = r * rot.cos();
        let z = r * rot.sin();

        let mut transform = Transform::default();
        transform.translation.x = x;
        transform.translation.y = CUBE_SIZE / 2.;
        transform.translation.z = z;
        let color = PLAYER_COLORS[handle % PLAYER_COLORS.len()];

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: CUBE_SIZE })),
                material: materials.add(color.into()),
                transform,
                ..default()
            },
            Player {
                handle,
                ..Default::default()
            },
            Score::default(),
            Velocity::default(),
            // this component indicates bevy_GGRS that parts of this entity should be saved and
            // loaded
            Rollback::new(rip.next_id()),
        ));
    }

    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(-4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    for mut transform in camera_query.iter_mut() {
        *transform = Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y);
    }
}

// Example system, manipulating a resource, will be added to the rollback schedule.
// Increases the frame count by 1 every update step. If loading and saving resources works
// correctly, you should see this resource rolling back, counting back up and finally increasing by
// 1 every update step
#[allow(dead_code)]
pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
    frame_count.frame += 1;
}

// Example system that moves the cubes, will be added to the rollback schedule.
// Filtering for the rollback component is a good way to make sure your game logic systems
// only mutate components that are being saved/loaded.
#[allow(dead_code)]
pub fn move_cube_system(
    mut query: Query<(&mut Transform, &mut Velocity, &Player), With<Rollback>>,
    inputs: Res<PlayerInputs<GGRSConfig>>,
) {
    for (mut t, mut v, p) in query.iter_mut() {
        let input = inputs[p.handle].0.inp;
        // set velocity through key presses
        if input & INPUT_UP != 0 && input & INPUT_DOWN == 0 {
            v.z -= MOVEMENT_SPEED;
        }
        if input & INPUT_UP == 0 && input & INPUT_DOWN != 0 {
            v.z += MOVEMENT_SPEED;
        }
        if input & INPUT_LEFT != 0 && input & INPUT_RIGHT == 0 {
            v.x -= MOVEMENT_SPEED;
        }
        if input & INPUT_LEFT == 0 && input & INPUT_RIGHT != 0 {
            v.x += MOVEMENT_SPEED;
        }

        // slow down
        if input & INPUT_UP == 0 && input & INPUT_DOWN == 0 {
            v.z *= FRICTION;
        }
        if input & INPUT_LEFT == 0 && input & INPUT_RIGHT == 0 {
            v.x *= FRICTION;
        }
        v.y *= FRICTION;

        // constrain velocity
        let mag = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
        if mag > MAX_SPEED {
            let factor = MAX_SPEED / mag;
            v.x *= factor;
            v.y *= factor;
            v.z *= factor;
        }

        // apply velocity
        t.translation.x += v.x;
        t.translation.y += v.y;
        t.translation.z += v.z;

        // constrain cube to plane
        t.translation.x = t.translation.x.max(-1. * (PLANE_SIZE - CUBE_SIZE) * 0.5);
        t.translation.x = t.translation.x.min((PLANE_SIZE - CUBE_SIZE) * 0.5);
        t.translation.z = t.translation.z.max(-1. * (PLANE_SIZE - CUBE_SIZE) * 0.5);
        t.translation.z = t.translation.z.min((PLANE_SIZE - CUBE_SIZE) * 0.5);
    }
}

pub fn move_enemy_system(
    mut enemy_query: Query<
        (&mut Transform, &mut Velocity, &Enemy),
        (With<Rollback>, Without<Player>),
    >,
    player_query: Query<(&Transform, &Score, &Player), (With<Rollback>, Without<Enemy>)>,
) {
    for (mut t, mut v, _) in enemy_query.iter_mut() {
        let enemy_position = t.translation;
        let mut highscore_beating_player_position = Vec3::default();
        let mut nearest_target_position = Vec3 {
            x: f32::INFINITY,
            y: f32::INFINITY,
            z: f32::INFINITY,
        };
        let mut is_highscore_beating_player = false;

        let mut top_score = 0;
        for (player_transform, player_score, _) in player_query.iter() {
            //get nearest player position
            let nearest_target_distance = enemy_position.distance(nearest_target_position);
            let player_distance = enemy_position.distance(player_transform.translation);
            if player_distance < nearest_target_distance {
                nearest_target_position = player_transform.translation;
            }
            //get "highscore beating" player position
            if player_score.current > player_score.highscore {
                highscore_beating_player_position = player_transform.translation;
                is_highscore_beating_player = true;
            }
            //set top_score
            if player_score.highscore > top_score {
                top_score = player_score.highscore;
            }
        }

        //position that the enemy will apply velocity toward
        let target_position = if is_highscore_beating_player {
            highscore_beating_player_position
        } else {
            nearest_target_position
        };

        //apply velocity
        let direction = (target_position - enemy_position).normalize_or_zero();
        let movement_vector = direction * MOVEMENT_SPEED;
        v.x += movement_vector.x;
        v.y += movement_vector.y;
        v.z += movement_vector.z;

        let mut speed_multiplier = 0.001 * top_score as f32;
        let speed_limit = 2.0;
        if speed_multiplier > speed_limit {
            speed_multiplier = speed_limit;
        }

        // constrain velocity
        let mag = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
        if mag > MAX_SPEED * speed_multiplier {
            let factor = (MAX_SPEED / mag) * speed_multiplier;
            v.x *= factor;
            v.y *= factor;
            v.z *= factor;
        }

        // apply velocity
        t.translation.x += v.x;
        t.translation.y += v.y;
        t.translation.z += v.z;

        // constrain enemy to plane
        t.translation.x = t.translation.x.max(-1. * (PLANE_SIZE - CUBE_SIZE) * 0.5);
        t.translation.x = t.translation.x.min((PLANE_SIZE - CUBE_SIZE) * 0.5);
        t.translation.z = t.translation.z.max(-1. * (PLANE_SIZE - CUBE_SIZE) * 0.5);
        t.translation.z = t.translation.z.min((PLANE_SIZE - CUBE_SIZE) * 0.5);
    }
}
pub struct ScoreboardScore {
    player_name: String,
    highscore: u32,
    score: u32,
}

pub fn update_scoreboard(
    mut text_query: Query<&mut Text, With<ScoreText>>,
    player_query: Query<(&Player, &Score)>,
) {
    // collect the scores and sort them
    let mut player_scores = Vec::<ScoreboardScore>::new();
    player_query.for_each(|(player, score)| {
        player_scores.push(ScoreboardScore {
            player_name: PLAYER_NAMES[player.handle % PLAYER_NAMES.len()].to_string(),
            highscore: score.highscore,
            score: score.current,
        });
    });
    player_scores.sort_by(|a, b| a.highscore.cmp(&b.highscore));
    player_scores.reverse();

    // create the scoreboard string
    let mut str = String::new();
    for player_score in player_scores {
        let player_name = player_score.player_name;
        let highscore = player_score.highscore;
        let score = player_score.score;
        str += &format!("{player_name}\nHighscore: {highscore}\nScore: {score}\n\n",);
    }
    text_query.single_mut().sections[0].value = str;
}

pub fn update_scores(
    mut player_query: Query<(&mut Score, &Transform), (With<Player>, Without<Enemy>)>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<Player>)>,
    frame_count: Res<FrameCount>,
) {
    //trigger death
    player_query.for_each_mut(|(mut score, player_transform)| {
        enemy_query.for_each(|enemy_transform| {
            let distance = player_transform
                .translation
                .distance(enemy_transform.translation);
            if distance <= ENEMY_RADIUS + CUBE_SIZE / 2. {
                score.last_death_frame = frame_count.frame;
            }
        });
    });

    //increment current score
    player_query.for_each_mut(|(mut score, _)| {
        score.current = frame_count.frame - score.last_death_frame;
        if score.current > score.highscore {
            score.highscore = score.current;
        }
    });
}

pub fn respawn_player(
    mut player_query: Query<(&Score, &mut Transform), (With<Player>, Without<Enemy>)>,
    frame_count: Res<FrameCount>,
) {
    player_query.for_each_mut(|(score, mut player_transform)| {
        //player died
        if score.current == 0 {
            //create a random position along a circle
            let radius = PLANE_SIZE / 2.;
            let possible_positions = 100;
            let random_value = (frame_count.frame % possible_positions) as f32;
            let rot = random_value as f32 / possible_positions as f32 * 2. * std::f32::consts::PI;
            let x = radius * rot.cos();
            let z = radius * rot.sin();

            //set the position
            player_transform.translation.x = x;
            player_transform.translation.y = CUBE_SIZE / 2.;
            player_transform.translation.z = z;
        }
    });
}
