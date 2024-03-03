use bevy::prelude::*;

use crate::states::MainState;

pub fn despawn_recursive_by_component<T: bevy::prelude::Component>(
    q: Query<Entity, With<T>>,
    mut commands: Commands,
) {
    for e in q.iter() {
        commands.entity(e).despawn_recursive();
    }
}
pub fn exit_to_menu_on_escape(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<MainState>>,
) {
    if input.just_released(KeyCode::Escape) {
        next_state.set(MainState::Menu);
    }
}
