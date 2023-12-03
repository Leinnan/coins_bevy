pub mod components;

use crate::consts;
use crate::game::components::*;
use crate::input::{AimingEndedEvent, AimingEvent, MainCamera};
use crate::states::MainState;
use crate::utils::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_rapier2d::prelude::Sensor;
use bevy_rapier2d::prelude::*;
#[derive(Event)]
pub struct GameProgressEvent;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameplaySettings>()
            .register_type::<GameplayProgress>()
            .init_resource::<GameplaySettings>()
            .init_resource::<GameplayProgress>()
            .register_type::<PlayerSpawnPoint>()
            .register_type::<EndPoint>()
            .register_type::<Obstacle>()
            .add_event::<GameProgressEvent>()
            .add_systems(OnEnter(MainState::Game), (setup_world, reset_progress))
            .add_systems(
                OnExit(MainState::Game),
                despawn_recursive_by_component::<GameRootObject>,
            )
            .add_systems(Startup, setup_graphics)
            .add_systems(
                PostUpdate,
                (display_events).run_if(in_state(MainState::Game)),
            )
            .add_systems(
                Update,
                (
                    add_elements,
                    arrow_display,
                    velocity_changed,
                    update_ui,
                    exit_to_menu_on_escape,
                )
                    .run_if(in_state(MainState::Game)),
            );
    }
}

fn add_elements(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    obstacles: Query<(Entity, &Transform, &Obstacle), Added<Obstacle>>,
    end_points: Query<(Entity, &Transform, &EndPoint), Added<EndPoint>>,
    start_point: Query<(Entity, &Transform), Added<PlayerSpawnPoint>>,
) {
    let candle_handle = asset_server.load("candle.png");
    for (e, transform, obstacle) in obstacles.iter() {
        commands
            .entity(e)
            .insert(Collider::ball(obstacle.radius))
            .insert(SpriteBundle {
                transform: *transform,
                texture: candle_handle.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(obstacle.radius * 2.0)),
                    ..default()
                },
                ..default()
            })
            .insert(Name::new("Candle".to_string()));
    }

    for (e, transform, end_point) in end_points.iter() {
        commands
            .entity(e)
            .insert((Collider::ball(end_point.radius - 50.0), Sensor))
            .insert(SpriteBundle {
                transform: *transform,
                texture: asset_server.load("end_circle.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(end_point.radius * 2.0)),
                    ..default()
                },
                ..default()
            })
            .insert(Name::new("Finish point"));
    }

    let radius = 20.0;
    for (e, transform) in start_point.iter() {
        commands
            .entity(e)
            .insert((
                RigidBody::Dynamic,
                Collider::ball(radius),
                ActiveEvents::COLLISION_EVENTS,
                ContactForceEventThreshold(10.0),
            ))
            .insert(Damping {
                linear_damping: 6.0,
                angular_damping: 9.0,
            })
            .insert(ZIndex::Global(2))
            .insert(PlayerControlled)
            .insert(GravityScale(0.0))
            .insert(Velocity::zero())
            .insert(ExternalImpulse {
                impulse: Vec2::new(0.0, 0.0),
                torque_impulse: 0.0,
            })
            .insert(Restitution::coefficient(0.95))
            .insert(SpriteBundle {
                transform: *transform,
                texture: asset_server.load("coin.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(radius * 2.0)),
                    ..default()
                },
                ..default()
            })
            .insert(Name::new("Player"));
    }
}

fn reset_progress(mut progress: ResMut<GameplayProgress>) {
    progress.reset();
}

fn setup_graphics(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

pub fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    let root = commands
        .spawn((
            GameRootObject,
            TransformBundle::default(),
            VisibilityBundle::default(),
        ))
        .id();
    commands.spawn((
        GameRootObject,
        TextBundle::from_section(
            "Press LPM to move",
            TextStyle {
                font: asset_server.load(consts::BASE_FONT),
                font_size: 25.0,
                color: consts::MY_ACCENT_COLOR,
            },
        )
        .with_text_alignment(TextAlignment::Left)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Px(15.0),
            ..default()
        }),
        TextChanges,
    ));

    let candle_radius = 45.0;
    for pos in [Vec2::new(0.0, -24.0), Vec2::new(100.0, -24.0)] {
        commands
            .spawn(TransformBundle {
                local: Transform::from_xyz(pos.x, pos.y, 0.0),
                ..Default::default()
            })
            .insert(Obstacle {
                radius: candle_radius,
            })
            .set_parent(root);
    }
    let end_circle_size = 80.0;
    commands
        .spawn((
            TransformBundle {
                local: Transform::from_xyz(45.0, -190.0, 0.0),
                ..default()
            },
            EndPoint {
                radius: end_circle_size,
            },
        ))
        .set_parent(root);

    commands
        .spawn((
            TransformBundle {
                local: Transform::from_xyz(0.0, 260.0, 0.0),
                ..default()
            },
            PlayerSpawnPoint,
        ))
        .set_parent(root);

    commands
        .spawn(SpriteBundle {
            transform: Transform::from_xyz(0.0, 260.0, 0.0).with_scale(Vec3::splat(0.0)),
            texture: asset_server.load("ornamented_arrow_alpha.png"),
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            ..default()
        })
        .set_parent(root)
        .insert(PointerArrow);
    commands
        .spawn(AudioBundle {
            source: asset_server.load("snd/spinning_tavern.ogg"),
            ..default()
        })
        .set_parent(root);
}

fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut ui_event: EventWriter<GameProgressEvent>,
    second_query: Query<&Sensor>,
    mut progress: ResMut<GameplayProgress>,
) {
    let mut should_send_event = false;
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(e, e2, _) => {
                let is_sensor = second_query.contains(*e) || second_query.contains(*e2);
                if !is_sensor {
                    progress.touches += 1;
                } else {
                    progress.is_inside_end_place = true;
                }
                should_send_event = true;
            }
            CollisionEvent::Stopped(e, e2, _) => {
                let is_sensor = second_query.contains(*e) || second_query.contains(*e2);
                if is_sensor {
                    progress.is_inside_end_place = false;
                }
                should_send_event = true;
            }
        }
    }
    if should_send_event {
        ui_event.send(GameProgressEvent);
    }
}

fn update_ui(mut query: Query<&mut Text, With<TextChanges>>, progress: Res<GameplayProgress>) {
    if query.is_empty() {
        return;
    }
    let mut text = query.single_mut();

    text.sections[0].value = format!(
        "Collisions: {}\nMoves: {}",
        progress.touches, progress.moves
    );
}

fn velocity_changed(
    query: Query<&Velocity, Changed<Velocity>>,
    mut ui_event: EventWriter<GameProgressEvent>,
    progress: Res<GameplayProgress>,
) {
    for velocity in &query {
        if velocity.linvel.length() < 0.1 {
            ui_event.send(GameProgressEvent);
            if progress.is_inside_end_place {
                eprintln!("WINNER!");
            }
        }
    }
}

fn arrow_display(
    mut arrow_q: Query<&mut Transform, With<PointerArrow>>,
    mut aim_event: EventReader<AimingEvent>,
    mut aim_event2: EventReader<AimingEndedEvent>,
    settings: Res<GameplaySettings>,
) {
    if arrow_q.is_empty() {
        return;
    }
    let mut transform = arrow_q.single_mut();
    for ev in aim_event.read() {
        transform.translation = Vec3::new(ev.player_pos.x, ev.player_pos.y, 0.0);
        transform.scale = Vec3::splat(ev.strength / settings.max_force.y * 0.6);
        transform.rotation = Quat::from_rotation_arc_2d(Vec2::new(0.0, 1.0), ev.direction);
    }
    for _ in aim_event2.read() {
        transform.scale = Vec3::splat(0.0);
    }
}
