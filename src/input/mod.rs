use crate::game::components::{GameplayProgress, GameplaySettings, PlayerControlled};

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

#[derive(Event)]
pub struct AimingEvent {
    pub player_pos: Vec2,
    pub direction: Vec2,
    pub strength: f32,
}

#[derive(Event)]
pub struct AimingEndedEvent {
    pub shoot: bool,
}

/// We store the world position of the mouse cursor here.
#[derive(Resource, Default, Reflect, Deref)]
#[reflect(Resource)]
pub struct MouseWorldPosition(Vec2);

pub struct GameInputPlugin;

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MouseWorldPosition>()
            .init_resource::<MouseWorldPosition>()
            .add_event::<AimingEndedEvent>()
            .add_event::<AimingEvent>()
            .add_systems(Update, player_input)
            .add_systems(Update, my_cursor_system);
    }
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

fn player_input(
    buttons: Res<Input<MouseButton>>,
    mouse_pos: Res<MouseWorldPosition>,
    settings: Res<GameplaySettings>,
    mut aim_event: EventWriter<AimingEvent>,
    mut aim_event_2: EventWriter<AimingEndedEvent>,
    mut ext_impulses: Query<(&mut ExternalImpulse, &Transform, &Velocity), With<PlayerControlled>>,
    mut progress: ResMut<GameplayProgress>,
) {
    if ext_impulses.is_empty() {
        return;
    }
    let released = buttons.just_released(MouseButton::Left);
    if buttons.pressed(MouseButton::Left) || released {
        let position = mouse_pos.0;
        let (mut external, transform, velocity) = ext_impulses.single_mut();

        if velocity.linvel.length() > 0.1 {
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
            progress.moves += 1;
        } else {
            aim_event.send(AimingEvent {
                player_pos,
                strength,
                direction: dir,
            });
        }
    }
}
