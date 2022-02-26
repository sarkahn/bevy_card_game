use bevy::prelude::*;
use bevy_ascii_terminal::Point2d;

use crate::GameState;

use super::{components::MapPosition, MapUnits};

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::BattleMap)
            .with_system(spawn_units))
        .add_system_set(
            SystemSet::on_update(GameState::BattleMap)
            .with_system(update_sprite_position)
            .with_system(update_map_units),
        );
    }
}

#[derive(Component, Default)]
pub struct MapUnit;

#[derive(Bundle)]
pub struct MapUnitBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    map_unit: MapUnit,
    pos: MapPosition,
}

fn make_map_unit(pos: impl Point2d, color: Color) -> MapUnitBundle {
    let sprite_bundle = SpriteBundle {
        sprite: Sprite {
            color: color,
            custom_size: Some(Vec2::ONE),
            ..Default::default()
        },
        ..Default::default()
    };
    MapUnitBundle {
        sprite_bundle,
        pos: pos.xy().into(),
        map_unit: Default::default()
    }
}

fn spawn_units(mut commands: Commands) {
    commands.spawn_bundle(make_map_unit([-5, -5], Color::RED));
    commands.spawn_bundle(make_map_unit([5, 5], Color::BLUE));
}

fn update_sprite_position(
    mut q_sprites: Query<(&mut Transform, &MapPosition), (Changed<MapPosition>, With<MapUnit>)>,
) {
    for (mut t, p) in q_sprites.iter_mut() {
        t.translation = p.xy.extend(5).as_vec3() + Vec3::new(0.5, 0.5, 0.0);
    }
}

fn update_map_units(
    mut units: ResMut<MapUnits>,
    q_moved_units: Query<(Entity,&MapPosition), (With<MapUnit>, Changed<MapPosition>)>,
    q_units: Query<(Entity, &MapPosition), With<MapUnit>>,
) {
    if q_moved_units.is_empty() {
        return;
    }
    units.clear();
    for (entity, pos) in q_units.iter() {
        units.set(pos.xy(), Some(entity));
    }
}
