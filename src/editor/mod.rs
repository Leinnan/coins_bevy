use crate::consts::*;
use crate::game::components::{EndPoint, Obstacle, PlayerSpawnPoint};
use crate::input::MouseWorldPosition;
use crate::{states::MainState, utils::exit_to_menu_on_escape};
use bevy::prelude::*;
use bevy::{
    app::{Plugin, Update},
    ecs::schedule::common_conditions::in_state,
};
use bevy_egui::{egui, EguiContext};

#[derive(Component, Default, Copy, Clone)]
pub struct EditorMapRoot;

#[derive(Component, Default, Copy, Clone)]
pub struct EditorObject;

#[derive(Component, Default, Debug, Copy, Clone, PartialEq)]
pub enum ActionToDo {
    MoveObject,
    SetPlayerSpawnPoint,
    SetEndPoint,
    AddObstacleToMap,
    RemoveObject,
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
                (inspector_ui, exit_to_menu_on_escape, draw_objects)
                    .run_if(in_state(MainState::Editor)),
            );
    }
}

fn startup(mut commands: Commands) {
    commands.spawn((
        EditorMapRoot,
        Name::new("MapEditor"),
        TransformBundle::default(),
    ));
}

fn draw_objects(
    mut gizmos: Gizmos,
    q: Query<&GlobalTransform, With<PlayerSpawnPoint>>,
    q2: Query<(&GlobalTransform, &EndPoint), With<EndPoint>>,
    q3: Query<(&GlobalTransform, &Obstacle), With<Obstacle>>,
) {
    for t in q.iter() {
        let t = t.translation();
        gizmos.circle_2d(Vec2::new(t.x, t.y), 10., MY_ACCENT_COLOR);
    }
    for (t, end_point) in q2.iter() {
        let t = t.translation();
        gizmos.circle_2d(Vec2::new(t.x, t.y), end_point.radius, Color::LIME_GREEN);
    }
    for (t, obstacle) in q3.iter() {
        let t = t.translation();
        gizmos.circle_2d(Vec2::new(t.x, t.y), obstacle.radius, Color::RED);
    }
}

fn inspector_ui(world: &mut World, mut enum_val: Local<ActionToDo>, mut ui_over: Local<bool>) {
    use bevy::window::PrimaryWindow;
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    let world_pos = **world
        .get_resource::<MouseWorldPosition>()
        .unwrap_or(&MouseWorldPosition::default());
    let world_root = world
        .query_filtered::<Entity, With<EditorMapRoot>>()
        .get_single(world)
        .unwrap();
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
            ui.horizontal(|ui| {
                for val in [
                    ActionToDo::SetPlayerSpawnPoint,
                    ActionToDo::AddObstacleToMap,
                    ActionToDo::SetEndPoint,
                    ActionToDo::MoveObject,
                    ActionToDo::RemoveObject,
                ] {
                    ui.radio_value(&mut *enum_val, val, format!("{:?}", val));
                }
            });
            ui.label(format!(
                "Mouse pos: {:.2}x{:.2}: {}",
                world_pos.x, world_pos.y, *ui_over
            ));
            *ui_over = ui.ui_contains_pointer();
            if world
                .get_resource::<Input<MouseButton>>()
                .unwrap()
                .just_released(MouseButton::Left)
                && !*ui_over
            {
                let transform = TransformBundle {
                    local: Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
                    ..default()
                };
                match *enum_val {
                    ActionToDo::SetPlayerSpawnPoint => {
                        world
                            .spawn((transform, PlayerSpawnPoint, EditorObject))
                            .set_parent(world_root);
                    }
                    ActionToDo::SetEndPoint => {
                        world
                            .spawn((transform, EndPoint { radius: 80.0 }, EditorObject))
                            .set_parent(world_root);
                    }
                    ActionToDo::AddObstacleToMap => {
                        world
                            .spawn((transform, Obstacle { radius: 45.0 }, EditorObject))
                            .set_parent(world_root);
                    }
                    ActionToDo::MoveObject => {
                        let e = get_closest_object_with_type::<EditorObject>(world)
                            .unwrap()
                            .0;
                        world.entity_mut(e).insert(transform);
                    }
                    ActionToDo::RemoveObject => {
                        let e = get_closest_object_with_type::<EditorObject>(world)
                            .unwrap()
                            .0;
                        world.entity_mut(e).despawn_recursive();
                    }
                    ActionToDo::DoNothing => {}
                }
            }
        });
}

pub fn get_closest_object_with_type<T: bevy::prelude::Component>(
    world: &mut World,
) -> Option<(Entity, &Transform)> {
    let world_pos = **world
        .get_resource::<MouseWorldPosition>()
        .unwrap_or(&MouseWorldPosition::default());
    let mouse_pos = Vec3::new(world_pos.x, world_pos.y, 0.0);
    let mut objects: Vec<(Entity, &Transform)> = world
        .query_filtered::<(Entity, &Transform), With<T>>()
        .iter(world)
        .collect();
    objects.sort_by(|a, b| {
        a.1.translation
            .distance(mouse_pos)
            .partial_cmp(&b.1.translation.distance(mouse_pos))
            .unwrap()
    });
    objects.first().copied()
}
