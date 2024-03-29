use crate::consts::*;
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::{
    bevy_inspector::hierarchy::SelectedEntities, DefaultInspectorConfigPlugin,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, git_info)
            .add_systems(
                Update,
                inspector_ui, //.run_if(not(in_state(crate::states::MainState::Editor))),
            )
            .add_plugins((EguiPlugin, DefaultInspectorConfigPlugin));
    }
}

fn git_info(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(
        TextBundle::from_section(
            format!("{} ( {} )", GIT_DATE, GIT_HASH),
            TextStyle {
                font: asset_server.load(BASE_FONT),
                font_size: 11.0,
                color: MY_ACCENT_COLOR,
            },
        )
        .with_text_justify(JustifyText::Right)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
    );
}

fn inspector_ui(
    world: &mut World,
    mut selected_entities: Local<SelectedEntities>,
    mut active: Local<bool>,
) {
    use bevy::window::PrimaryWindow;
    if world
        .get_resource::<ButtonInput<KeyCode>>()
        .unwrap()
        .just_released(KeyCode::F1)
    {
        *active = !*active;
    }
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    if !*active {
        return;
    }
    egui::SidePanel::left("hierarchy")
        .default_width(200.0)
        .show_animated(egui_context.get_mut(), true, |ui| {
            ui.add_space(10.0);
            ui.heading(
                egui::RichText::new("Hierarchy")
                    .strong()
                    .color(MY_ACCENT_COLOR32),
            );
            ui.label(egui::RichText::new("Press F1 to toggle UI").small());
            ui.add_space(15.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui(
                    world,
                    ui,
                    &mut selected_entities,
                );
                ui.label(
                    egui::RichText::new("Resources")
                        .strong()
                        .color(MY_ACCENT_COLOR32),
                );
                bevy_inspector_egui::bevy_inspector::ui_for_resources(world, ui);

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
