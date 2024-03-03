use crate::consts::*;
use crate::game::components::{EndPoint, Obstacle, PlayerSpawnPoint};
use crate::input::MouseWorldPosition;
use crate::{states::MainState, utils::exit_to_menu_on_escape};
use bevy::prelude::*;
use bevy::{
    app::{Plugin, Update},
    ecs::schedule::common_conditions::in_state,
    tasks::IoTaskPool,
};
use bevy_egui::{egui, EguiContext};
use std::{fs::File, io::Write};

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
                (
                    inspector_ui,
                    exit_to_menu_on_escape,
                    draw_objects,
                    add_missing_info,
                )
                    .chain()
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

fn add_missing_info(
    q: Query<&Children, (With<EditorMapRoot>, Without<EditorObject>)>,
    q2: Query<Entity, (With<EditorObject>, Without<GlobalTransform>)>,
    mut commands: Commands,
) {
    for c in q.iter() {
        for e in c.iter() {
            if let Some(mut entity_cmd) = commands.get_entity(*e) {
                entity_cmd.insert(EditorObject);
            }
        }
    }
    for e in q2.iter() {
        commands.entity(e).insert(GlobalTransform::default());
    }
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

fn inspector_ui(
    world: &mut World,
    mut enum_val: Local<ActionToDo>,
    mut ui_over: Local<bool>,
    mut filename: Local<String>,
) {
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
            let mut text = (*filename).clone();
            ui.text_edit_singleline(&mut text);
            *filename = text;
            if ui.button("Save map").clicked() {
                save_map(world, "01.scn.ron".into());
            }
            if ui.button("Load map").clicked() {
                load_map(world, "01.scn.ron".into());
            }
            *ui_over = ui.ui_contains_pointer();
            if world
                .get_resource::<ButtonInput<MouseButton>>()
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
                        if let Some(e) = get_closest_object_with_type::<PlayerSpawnPoint>(world) {
                            world.entity_mut(e).insert(transform);
                        } else {
                            world
                                .spawn((transform, PlayerSpawnPoint))
                                .set_parent(world_root);
                        }
                    }
                    ActionToDo::SetEndPoint => {
                        if let Some(e) = get_closest_object_with_type::<EndPoint>(world) {
                            world.entity_mut(e).insert(transform);
                        } else {
                            world
                                .spawn((transform, EndPoint { radius: 80.0 }))
                                .set_parent(world_root);
                        }
                    }
                    ActionToDo::AddObstacleToMap => {
                        world
                            .spawn((transform, Obstacle { radius: 45.0 }))
                            .set_parent(world_root);
                    }
                    ActionToDo::MoveObject => {
                        if let Some(e) = get_closest_object_with_type::<EditorObject>(world) {
                            world.entity_mut(e).insert(transform);
                        }
                    }
                    ActionToDo::RemoveObject => {
                        if let Some(e) = get_closest_object_with_type::<EditorObject>(world) {
                            world.entity_mut(e).despawn_recursive();
                        }
                    }
                    ActionToDo::DoNothing => {}
                }
            }
        });
}

pub fn get_closest_object_with_type<T: bevy::prelude::Component>(
    world: &mut World,
) -> Option<Entity> {
    let world_pos = **world
        .get_resource::<MouseWorldPosition>()
        .unwrap_or(&MouseWorldPosition::default());
    let mouse_pos = Vec3::new(world_pos.x, world_pos.y, 0.0);
    let mut objects: Vec<(Entity, &Transform)> = world
        .query_filtered::<(Entity, &Transform), With<T>>()
        .iter(world)
        .collect();
    if objects.is_empty() {
        return None;
    }
    objects.sort_by(|a, b| {
        a.1.translation
            .distance(mouse_pos)
            .partial_cmp(&b.1.translation.distance(mouse_pos))
            .unwrap()
    });

    Some(objects.first().unwrap().0)
}

pub fn save_map(world: &mut World, filename: String) {
    let mut scene_world = World::new();
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    scene_world.insert_resource(type_registry);

    for (e, t) in world
        .query_filtered::<(Entity, &Transform), With<EditorObject>>()
        .iter(world)
    {
        let id = scene_world.spawn(*t).id();
        let mut entity_mut = scene_world.entity_mut(id);
        if world.entity(e).contains::<PlayerSpawnPoint>() {
            entity_mut.insert(PlayerSpawnPoint);
        }
        if let Some(obj) = world.entity(e).get::<EndPoint>() {
            entity_mut.insert(obj.clone());
        }
        if let Some(obj) = world.entity(e).get::<Obstacle>() {
            entity_mut.insert(obj.clone());
        }
    }

    let scene = DynamicScene::from_world(&scene_world);
    let type_registry = scene_world.resource::<AppTypeRegistry>();
    let serialized_scene = scene.serialize_ron(type_registry).unwrap();

    // Showing the scene in the console
    info!("{}", serialized_scene);

    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get()
        .spawn(async move {
            // Write the scene RON data to file
            File::create(format!("assets/scenes/{filename}"))
                .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                .expect("Error while writing scene to file");
        })
        .detach();
}

fn load_map(world: &mut World, filename: String) {
    let world_root = world
        .query_filtered::<Entity, With<EditorMapRoot>>()
        .get_single(world)
        .unwrap();

    world
        .get_entity_mut(world_root)
        .unwrap()
        .despawn_recursive();

    let scene = world
        .get_resource_mut::<AssetServer>()
        .unwrap()
        .load(format!("scenes/{filename}"));
    world.spawn((
        DynamicSceneBundle {
            // Scenes are loaded just like any other asset.
            scene,
            ..default()
        },
        Name::new("MapEditor"),
        EditorMapRoot,
    ));
}
