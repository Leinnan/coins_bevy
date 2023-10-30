mod consts;
mod debug;
mod game;
mod input;
mod states;

use crate::game::GamePlugin;
use crate::input::GameInputPlugin;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.09, 0.09, 0.13)))
        .insert_resource(Msaa::Off)
        .add_state::<states::MainState>()
        .add_plugins((
            DefaultPlugins,
            debug::DebugPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
            GameInputPlugin,
            GamePlugin,
        ))
        .run();
}
