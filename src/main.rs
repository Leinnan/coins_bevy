mod debug;
mod consts;

use bevy::prelude::*;
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

#[derive(Resource, Reflect)]
pub struct GameplaySettings
{
    pub min_force : Vec2,
    pub max_force : Vec2
}

impl GameplaySettings{
    pub fn get_shoot_strength(&self, distance: f32) -> Option<f32> {
        if distance < self.min_force.x {
            return None;
        }
        let strength = distance.min(self.max_force.x) / self.max_force.x * (self.max_force.y - self.min_force.y) + self.min_force.y;
        eprintln!("Distance {distance} with strength: {strength}");
        Some(strength)
    }
}

impl Default for GameplaySettings {
    fn default() -> Self {
        GameplaySettings{
            min_force: Vec2::new(25.0,1.0),
            max_force: Vec2::new(150.0,125.0)
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.09, 0.09, 0.13)))
        .insert_resource(Msaa::Off)
        .register_type::<GameplaySettings>()
        .register_type::<MouseWorldPosition>()
        .init_resource::<MouseWorldPosition>()
        .init_resource::<GameplaySettings>()
        .add_plugins((
            DefaultPlugins,
            debug::DebugPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, (setup_graphics, setup_physics))
        .add_systems(PostUpdate, display_events)
        .add_systems(Update, (my_cursor_system,player_input))
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
    if let Some(world_position) = window.cursor_position()
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
            "Press space to spawn ",
            TextStyle {
                font: asset_server.load(consts::BASE_FONT),
                font_size: 15.0,
                color: consts::MY_ACCENT_COLOR,
            },
        )
            .with_text_alignment(TextAlignment::Center)
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
) {
    for mut text in &mut query {
        for collision_event in collision_events.iter() {
            text.sections[0].value = format!("Collision event: {collision_event:?}");
        }

        for contact_force_event in contact_force_events.iter() {
            text.sections[0].value =
                format!("Contact force event: {contact_force_event:?}");
        }
    }
}

pub fn setup_physics(mut commands: Commands, asset_server: Res<AssetServer>) {
    let candle_radius = 45.0;
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, -24.0, 0.0)),
        Collider::ball(candle_radius),
    )).insert(SpriteBundle{
        texture: asset_server.load("candle.png"),
        sprite: Sprite{
            custom_size: Some(Vec2::splat(candle_radius * 2.0)),
            ..default()
        },
        ..default()
    });

    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, 100.0, 0.0)),
        Collider::cuboid(80.0, 30.0),
        Sensor,
    ));
    let radius = 20.0;
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, 260.0, 0.0)),
        RigidBody::Dynamic,
        Collider::ball(radius),
        ActiveEvents::COLLISION_EVENTS,
        ContactForceEventThreshold(10.0),
    )).insert(Damping { linear_damping: 8.0, angular_damping: 8.0 }).insert(PlayerControlled).insert(GravityScale(0.0)).insert(ExternalImpulse {
        impulse: Vec2::new(0.0, 0.0),
        torque_impulse: 0.0,
    }).insert(Restitution::coefficient(0.95)).insert(SpriteBundle{
        texture: asset_server.load("coin.png"),
        sprite: Sprite{
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        },
        ..default()
    });
}

fn player_input(buttons: Res<Input<MouseButton>>, mycoords: Res<MouseWorldPosition>, settings: Res<GameplaySettings>, mut ext_impulses: Query<(&mut ExternalImpulse, &Transform), With<PlayerControlled>>) {
    if buttons.just_pressed(MouseButton::Left) {
        let position = mycoords.0.clone();

        for (mut external, transform) in ext_impulses.iter_mut() {
            let vec2 = Vec2::new(transform.translation.x,transform.translation.y);
            let distance = position.distance(vec2);
            let strength = settings.get_shoot_strength(distance);
            if strength.is_none() {
                continue;
            }
            let strength = strength.unwrap();
            let dir = (position - vec2).normalize();
            eprintln!("{},{},{},{}",position,vec2,dir,strength);
            external.impulse = dir * strength;
            external.torque_impulse = 0.3;
        }
    }
}
