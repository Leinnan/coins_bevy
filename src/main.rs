mod consts;
mod debug;
mod editor;
mod game;
mod input;
mod states;
pub mod utils;

use crate::game::GamePlugin;
use crate::input::GameInputPlugin;
use crate::states::GameStatesPlugin;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use editor::MapEditorPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.09, 0.09, 0.13)))
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            debug::DebugPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
            GameInputPlugin,
            GamePlugin,
            GameStatesPlugin,
            MapEditorPlugin,
        ))
        .run();
}
