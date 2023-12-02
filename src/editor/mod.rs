use crate::consts::*;
use crate::{states::MainState, utils::exit_to_menu_on_escape};
use bevy::prelude::*;
use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{common_conditions::in_state, IntoSystemConfigs},
};
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;

#[derive(Component, Default, Copy, Clone)]
pub struct EditorMapRoot;

#[derive(Component, Default, Copy, Clone)]
pub struct PlayerSpawnPoint;

#[derive(Component, Default, Copy, Clone)]
pub struct EndPoint;

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
                (inspector_ui, exit_to_menu_on_escape).run_if(in_state(MainState::Editor)),
            );
    }
}

fn startup(mut commands: Commands) {
    commands.spawn((EditorMapRoot, Name::new("MapEditor")));
}

fn inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {
    use bevy::window::PrimaryWindow;
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    egui::SidePanel::left("Editor")
        .default_width(200.0)
        .show_animated(egui_context.get_mut(), true, |ui| {
            ui.add_space(10.0);
            ui.heading(
                egui::RichText::new("Map Editor")
                    .strong()
                    .color(MY_ACCENT_COLOR32),
            );
            ui.add_space(15.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui(
                    world,
                    ui,
                    &mut selected_entities,
                );

                ui.allocate_space(ui.available_size());
            });
        });

    egui::SidePanel::right("inspector")
        .default_width(250.0)
        .show(egui_context.get_mut(), |ui| {
            ui.add_space(10.0);
            ui.heading(
                egui::RichText::new("Inspector")
                    .heading()
                    .strong()
                    .color(MY_ACCENT_COLOR32),
            );
            ui.label(
                egui::RichText::new(format!("{} ( {} )", GIT_DATE, GIT_HASH)).small(), // .weak(),
            );
            ui.add_space(15.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                match selected_entities.as_slice() {
                    &[entity] => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
                    }
                    entities => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
                            world, entities, ui,
                        );
                    }
                }

                ui.allocate_space(ui.available_size());
            });
        });
}
