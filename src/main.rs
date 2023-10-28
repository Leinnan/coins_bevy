mod debug;
mod consts;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
struct TextChanges;

#[derive(Component)]
struct PlayerControlled;

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.09, 0.09, 0.13)))
        .insert_resource(Msaa::Off)
        .init_resource::<MyWorldCoords>()
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
    mut mycoords: ResMut<MyWorldCoords>,
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

pub fn setup_physics(mut commands: Commands) {
    /*
     * Ground
     */
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, -24.0, 0.0)),
        Collider::cuboid(80.0, 20.0),
    ));

    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, 100.0, 0.0)),
        Collider::cuboid(80.0, 30.0),
        Sensor,
    ));
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, 260.0, 0.0)),
        RigidBody::Dynamic,
        Collider::ball(11.0),
        ActiveEvents::COLLISION_EVENTS,
        ContactForceEventThreshold(10.0),
    )).insert(Damping { linear_damping: 15.0, angular_damping: 1.0 }).insert(PlayerControlled).insert(GravityScale(0.0)).insert(ExternalImpulse {
        impulse: Vec2::new(0.0, -0.0),
        torque_impulse: 0.0,
    });
}

fn player_input(buttons: Res<Input<MouseButton>>, mycoords: Res<MyWorldCoords>,mut ext_impulses: Query<(&mut ExternalImpulse, &Transform), With<PlayerControlled>>) {
    if buttons.just_pressed(MouseButton::Left) {
        let position = mycoords.0.clone();

        for (mut external, transform) in ext_impulses.iter_mut() {
            let vec2 = Vec2::new(transform.translation.x,transform.translation.y);
            let strength = position.distance(vec2).min(200.0) / 3.0;
            let dir = (position - vec2).normalize();
            eprintln!("{},{},{},{}",position,vec2,dir,strength);
            external.impulse = dir * strength;
            external.torque_impulse = 0.0;
        }
    }
}
