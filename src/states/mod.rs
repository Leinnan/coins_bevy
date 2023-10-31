mod menu;

use crate::states::menu::MenuPlugin;
use bevy_button_released_plugin::*;
use bevy::prelude::*;

#[derive(Clone, Debug, Default, Hash, Eq, States, PartialEq)]
pub enum MainState {
    #[default]
    Menu,
    Game,
}

pub struct GameStatesPlugin;

impl Plugin for GameStatesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MenuPlugin,ButtonsReleasedPlugin)).add_state::<MainState>();
    }
}
