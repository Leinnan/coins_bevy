use crate::consts::*;
use crate::input::MouseWorldPosition;
use crate::{states::MainState, utils::exit_to_menu_on_escape};
use bevy::prelude::*;
use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{common_conditions::in_state, IntoSystemConfigs},
};
use bevy_egui::{egui, EguiContext};

#[derive(Component, Default, Copy, Clone)]
pub struct EditorMapRoot;

#[derive(Component, Default, Copy, Clone)]
pub struct PlayerSpawnPoint;

#[derive(Component, Default, Copy, Clone)]
pub struct EndPoint
{
    pub radius: f32,
}

#[derive(Component, Default, Copy, Clone)]
pub enum ActionToDo{
    SetPlayerSpawnPoint(Vec2),
    SetEndPoint(Vec2),
    AddObstacleToMap(Vec2),
    #[default]
    DoNothing,
}

pub struct MapEditorPlugin;

impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(OnEnter(MainState::Editor), startup)
            .add_systems(
                OnExit(MainState::Editor),
                crate::utils::despawn_recursive_by_component::<EditorMapRoot>,
            )
            .add_systems(
                Update,
                (inspector_ui, exit_to_menu_on_escape,draw_objects).run_if(in_state(MainState::Editor)),
            );
    }
}

fn startup(mut commands: Commands) {
    commands.spawn((EditorMapRoot, Name::new("MapEditor"),TransformBundle::default()));
}

fn draw_objects(mut gizmos: Gizmos, q: Query<&GlobalTransform, With<PlayerSpawnPoint>>,q2: Query<(&GlobalTransform,&EndPoint), With<EndPoint>>) {
    for t in q.iter() {
        let t = t.translation();
        gizmos.circle_2d(Vec2::new(t.x, t.y), 10., MY_ACCENT_COLOR);
    }    
    for (t,end_point) in q2.iter() {
        let t = t.translation();
        gizmos.circle_2d(Vec2::new(t.x, t.y), end_point.radius, MY_ACCENT_COLOR);
    }
}

fn inspector_ui(world: &mut World, mut enum_val : Local<ActionToDo>) {
    use bevy::window::PrimaryWindow;
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    let world_pos = **world.get_resource::<MouseWorldPosition>().unwrap_or(&MouseWorldPosition::default());
    egui::TopBottomPanel::bottom("LevelEditor")
        .default_height(300.0)
        .show(egui_context.get_mut(), |ui| {
            ui.add_space(10.0);
            ui.heading(
                egui::RichText::new("Map Editor")
                    .strong()
                    .color(MY_ACCENT_COLOR32),
            );
            ui.add_space(15.0);
            let ui_over = ui.ui_contains_pointer();
            // ui.horizontal(|ui| {
            //     ui.radio_value(&mut enum_val, ActionToDo::DoNothing, "First");
            //     ui.radio_value(&mut enum_val, ActionToDo::SetPlayerSpawnPoint(world_pos), "Second");
            //     ui.radio_value(&mut enum_val, ActionToDo::DoNothing, "Third");
            // });
            ui.label(format!("Mouse pos: {:.2}x{:.2}: {}",world_pos.x,world_pos.y,ui_over));
        });

}
