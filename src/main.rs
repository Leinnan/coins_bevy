mod debug;
mod consts;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
struct TextChanges;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.09, 0.09, 0.13)))
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            debug::DebugPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, (setup_graphics, setup_physics))
        .add_systems(PostUpdate, display_events)
        .add_systems(Update, player_input)
        .run();
}

fn setup_graphics(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
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
}

fn player_input(keys: ResMut<Input<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Space) {
        commands.spawn((
            TransformBundle::from(Transform::from_xyz(0.0, 260.0, 0.0)),
            RigidBody::Dynamic,
            Collider::cuboid(10.0, 10.0),
            ActiveEvents::COLLISION_EVENTS,
            ContactForceEventThreshold(10.0),
        ));
    }
}
