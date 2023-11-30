use bevy::prelude::*;

use crate::states::MainState;

pub fn despawn_recursive_by_component<T: bevy::prelude::Component>(
    q: Query<Entity, With<T>>,
    mut commands: Commands,
) {
    let e = q.single();
    commands.entity(e).despawn_recursive();
}
pub fn exit_to_menu_on_escape(
    input: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<MainState>>,
) {
    if input.just_released(KeyCode::Escape) {
        next_state.set(MainState::Menu);
    }
}
