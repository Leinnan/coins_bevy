use bevy::prelude::*;
use bevy::prelude::{Component, Resource};

#[derive(Component, Default, Copy, Clone)]
pub struct GameRootObject;

#[derive(Component)]
pub struct TextChanges;

// Used to help identify coin controlled by player
#[derive(Component)]
pub struct PlayerControlled;

#[derive(Component)]
pub struct PointerArrow;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct GameplaySettings {
    pub min_force: Vec2,
    pub max_force: Vec2,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct GameplayProgress {
    pub touches: i32,
    pub moves: i32,
    pub is_inside_end_place: bool,
}

impl GameplaySettings {
    pub fn get_shoot_strength(&self, distance: f32) -> Option<f32> {
        if distance < self.min_force.x {
            return None;
        }
        let strength = distance.min(self.max_force.x) / self.max_force.x
            * (self.max_force.y - self.min_force.y)
            + self.min_force.y;
        eprintln!("Distance {distance} with strength: {strength}");
        Some(strength)
    }
}

impl Default for GameplaySettings {
    fn default() -> Self {
        GameplaySettings {
            min_force: Vec2::new(25.0, 1.0),
            max_force: Vec2::new(150.0, 200.0),
        }
    }
}
