mod consts;
mod debug;

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
struct TextChanges;

// Used to help identify coin controlled by player
#[derive(Component)]
struct PlayerControlled;

/// We store the world position of the mouse cursor here.
#[derive(Resource, Default, Reflect)]
struct MouseWorldPosition(Vec2);

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct PointerArrow;

#[derive(Resource, Reflect)]
pub struct GameplaySettings {
    pub min_force: Vec2,
    pub max_force: Vec2,
}

#[derive(Resource, Reflect, Default)]
pub struct GameplayProgress {
    pub touches: i32,
    pub moves: i32,
}

#[derive(Event)]
struct AimingEvent {
    pub player_pos: Vec2,
    pub direction: Vec2,
    pub strength: f32,
}

#[derive(Event)]
struct AimingEndedEvent {
    pub shoot: bool,
}

impl GameplaySettings {
    pub fn get_shoot_strength(&self, distance: f32) -> Option<f32> {
        if distance < self.min_force.x {
            return None;
        }
        let strength = distance.min(self.max_force.x) / self.max_force.x
            * (self.max_force.y - self.min_force.y)
            + self.min_force.y;
        eprintln!("Distance {distance} with strength: {strength}");
        Some(strength)
    }
}

impl Default for GameplaySettings {
    fn default() -> Self {
        GameplaySettings {
            min_force: Vec2::new(25.0, 1.0),
            max_force: Vec2::new(150.0, 125.0),
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.09, 0.09, 0.13)))
        .insert_resource(Msaa::Off)
        .register_type::<GameplaySettings>()
        .register_type::<MouseWorldPosition>()
        .register_type::<GameplayProgress>()
        .init_resource::<MouseWorldPosition>()
        .init_resource::<GameplaySettings>()
        .init_resource::<GameplayProgress>()
        .add_event::<AimingEvent>()
        .add_event::<AimingEndedEvent>()
        .add_plugins((
            DefaultPlugins,
            debug::DebugPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, (setup_graphics, setup_physics))
        .add_systems(PostUpdate, display_events)
        .add_systems(
            Update,
            (
                my_cursor_system,
                player_input,
                arrow_display,
                velocity_changed,
            ),
        )
        .run();
}
fn my_cursor_system(
    mut mycoords: ResMut<MouseWorldPosition>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
    }
}

fn setup_graphics(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
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
}

fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
    mut query: Query<&mut Text, With<TextChanges>>,
    mut progress: ResMut<GameplayProgress>,
) {
    for mut text in &mut query {
        for collision_event in collision_events.iter() {
            match collision_event {
                CollisionEvent::Started(_, _, _) => progress.touches = progress.touches + 1,
                CollisionEvent::Stopped(_, _, _) => {}
            }
            text.sections[0].value = format!(
                "Collisions: {}\nMoves: {}",
                progress.touches, progress.moves
            );
        }

        for contact_force_event in contact_force_events.iter() {
            text.sections[0].value = format!("Contact force event: {contact_force_event:?}");
        }
    }
}

pub fn setup_physics(mut commands: Commands, asset_server: Res<AssetServer>) {
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

    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, 100.0, 0.0)),
        Collider::cuboid(80.0, 30.0),
        Sensor,
    ));
    let radius = 20.0;
    commands
        .spawn((
            RigidBody::Dynamic,
            Collider::ball(radius),
            ActiveEvents::COLLISION_EVENTS,
            ContactForceEventThreshold(10.0),
        ))
        .insert(Damping {
            linear_damping: 8.0,
            angular_damping: 8.0,
        })
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
}

fn velocity_changed(query: Query<&Velocity, Changed<Velocity>>) {
    for velocity in &query {
        if velocity.linvel.length() > 0.1 {
            eprintln!("{:?} velocity", velocity);
        }
    }
}

fn player_input(
    buttons: Res<Input<MouseButton>>,
    mouse_pos: Res<MouseWorldPosition>,
    settings: Res<GameplaySettings>,
    mut aim_event: EventWriter<AimingEvent>,
    mut aim_event_2: EventWriter<AimingEndedEvent>,
    mut ext_impulses: Query<(&mut ExternalImpulse, &Transform, &Velocity), With<PlayerControlled>>,
    mut progress: ResMut<GameplayProgress>,
) {
    let released = buttons.just_released(MouseButton::Left);
    if buttons.pressed(MouseButton::Left) || released {
        let position = mouse_pos.0;
        let (mut external, transform, velocity) = ext_impulses.single_mut();

        if velocity.linvel.length() > 0.1 {
            eprintln!("Still moving({}), skipping", velocity.linvel);
            if released {
                aim_event_2.send(AimingEndedEvent { shoot: false });
            }
            return;
        }

        let player_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let distance = position.distance(player_pos);
        let strength = settings.get_shoot_strength(distance);
        if strength.is_none() {
            if released {
                aim_event_2.send(AimingEndedEvent { shoot: false });
            }
            return;
        }
        let strength = strength.unwrap();
        let dir = (position - player_pos).normalize();
        if released {
            eprintln!("{},{},{},{}", position, player_pos, dir, strength);
            aim_event_2.send(AimingEndedEvent { shoot: true });
            external.impulse = dir * strength;
            external.torque_impulse = 0.3;
            progress.moves = progress.moves + 1;
        } else {
            aim_event.send(AimingEvent {
                player_pos,
                strength,
                direction: dir,
            });
        }
    }
}

fn arrow_display(
    mut arrow_q: Query<&mut Transform, With<PointerArrow>>,
    mut aim_event: EventReader<AimingEvent>,
    mut aim_event2: EventReader<AimingEndedEvent>,
    settings: Res<GameplaySettings>,
) {
    let mut transform = arrow_q.single_mut();
    for ev in aim_event.iter() {
        transform.translation = Vec3::new(ev.player_pos.x, ev.player_pos.y, 0.0);
        transform.scale = Vec3::splat(ev.strength / settings.max_force.y * 0.6);
        transform.rotation = Quat::from_rotation_arc_2d(Vec2::new(0.0, 1.0), ev.direction);
    }
    for ev in aim_event2.iter() {
        transform.scale = Vec3::splat(0.0);
    }
}
