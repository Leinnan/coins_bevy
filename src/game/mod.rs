pub mod components;

use crate::consts;
use crate::game::components::*;
use crate::input::{AimingEndedEvent, AimingEvent, MainCamera};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_rapier2d::prelude::Sensor;
use bevy_rapier2d::prelude::*;
use crate::states::MainState;

#[derive(Event)]
pub struct GameProgressEvent;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameplaySettings>()
            .register_type::<GameplayProgress>()
            .init_resource::<GameplaySettings>()
            .init_resource::<GameplayProgress>()
            .add_event::<GameProgressEvent>()
            .add_systems(OnEnter(MainState::Game), setup_physics)
            .add_systems(Startup, setup_graphics)
            .add_systems(PostUpdate, (display_events).run_if(in_state(MainState::Game)))
            .add_systems(Update, (arrow_display, velocity_changed, update_ui).run_if(in_state(MainState::Game)));
    }
}
fn setup_graphics(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

pub fn setup_physics(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TextBundle::from_section(
            "Press LPM to move",
            TextStyle {
                font: asset_server.load(consts::BASE_FONT),
                font_size: 15.0,
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
    let candle_handle = asset_server.load("candle.png");
    for pos in [Vec2::new(0.0, -24.0), Vec2::new(100.0, -24.0)] {
        commands
            .spawn(Collider::ball(candle_radius))
            .insert(SpriteBundle {
                transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                texture: candle_handle.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(candle_radius * 2.0)),
                    ..default()
                },
                ..default()
            })
            .insert(Name::new(format!("Candle {}x{}", pos.x, pos.y)));
    }
    let end_circle_size = 80.0;
    commands
        .spawn((Collider::ball(end_circle_size - 50.0), Sensor))
        .insert(SpriteBundle {
            transform: Transform::from_xyz(45.0, -190.0, 0.0),
            texture: asset_server.load("end_circle.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(end_circle_size * 2.0)),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Finish point"));
    let radius = 20.0;
    commands
        .spawn((
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
            transform: Transform::from_xyz(0.0, 260.0, 0.0),
            texture: asset_server.load("coin.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::splat(radius * 2.0)),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Player"));

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
        .insert(PointerArrow);
    commands.spawn(AudioBundle {
        source: asset_server.load("snd/spinning_tavern.ogg"),
        ..default()
    });
}

fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut ui_event: EventWriter<GameProgressEvent>,
    second_query: Query<&Sensor>,
    mut progress: ResMut<GameplayProgress>,
) {
    let mut should_send_event = false;
    for collision_event in collision_events.iter() {
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
    if query.is_empty(){
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
    if arrow_q.is_empty(){
        return;
    }
    let mut transform = arrow_q.single_mut();
    for ev in aim_event.iter() {
        transform.translation = Vec3::new(ev.player_pos.x, ev.player_pos.y, 0.0);
        transform.scale = Vec3::splat(ev.strength / settings.max_force.y * 0.6);
        transform.rotation = Quat::from_rotation_arc_2d(Vec2::new(0.0, 1.0), ev.direction);
    }
    for _ in aim_event2.iter() {
        transform.scale = Vec3::splat(0.0);
    }
}
