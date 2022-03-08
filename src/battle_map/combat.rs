use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_egui::{egui::{self, FontDefinitions}, EguiContext};

use crate::{
    arena::{ArenaCombat, ArenaState},
    config::ConfigAsset,
    ldtk_loader::LdtkMap,
    make_sprite, GameState, GridHelper, SETTINGS_PATH, TILE_SIZE, screen_to_world, make_spritesheet_bundle,
};

use super::{
    map::CollisionMap,
    spawn::{
        DespawnTimer, 
        //SpawnEntity
    },
    MapUnits, EnemyUnit,
};

pub struct MapCombatPlugin;

impl Plugin for MapCombatPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system_set(
            SystemSet::on_update(GameState::BattleMap)
            .with_system(on_collision))
        .add_system_set(
            SystemSet::on_update(GameState::BeginningCombat)
            .with_system(begin_combat),
        );
    }
}

fn on_collision(
    mut commands: Commands,
    q_enemies: Query<(Entity, &Transform), (With<EnemyUnit>, Changed<Transform>)>,
    units: Res<MapUnits>,
    mut state: ResMut<State<GameState>>,
    config: Res<Assets<ConfigAsset>>,
    ldtk: Res<Assets<LdtkMap>>,
    asset_server: Res<AssetServer>,
) {
    if let Some(config) = config.get(SETTINGS_PATH) {
        if let Some(ldtk) = ldtk.get(&config.settings.map_file) {
            for (enemy, transform) in q_enemies.iter() {
                let xy = units.xy_to_grid(transform.translation.xy());
                if let Some(player) = units.get_from_grid_xy(xy) {
                    let mut pos = transform.translation;
                    pos += Vec3::new(0.0, 0.0, 1.0) * TILE_SIZE as f32;
                    let mut text_pos = Vec3::new(0.0, 1.0, 0.0) * TILE_SIZE as f32;
                    text_pos.z += 1.0;

                    let fight_entity = ldtk.entity_defs().get_tagged("begin_combat").next().unwrap();
                    let atlas = ldtk.tileset_from_id(fight_entity.tileset_id().unwrap());
                    let atlas = atlas.unwrap().atlas();

                    let sprite = make_spritesheet_bundle(
                        fight_entity.tile_id().unwrap() as usize, atlas.clone(), pos
                    );
                    let text = Text2dBundle {
                        text: Text::with_section(
                            "FIGHT IT OUT!", 
                            TextStyle {
                                font: asset_server.load("fonts/DejaVuSerif-Bold.ttf"),
                                font_size: 30.0,
                                color: Color::WHITE,
                            }, 
                            TextAlignment::default()),
                        transform: Transform::from_translation(text_pos),
                        ..Default::default()
                    };
                    let text = commands.spawn_bundle(text).id();
                    commands.spawn_bundle(sprite)
                    .insert(DespawnTimer::new(3.0))
                    .insert(BeginCombat { player_party: player, enemy_party: enemy })
                    .add_child(text)
                    ;

                    state.set(GameState::BeginningCombat).unwrap();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct BeginCombat {
    player_party: Entity,
    enemy_party: Entity,
}

#[derive(Component)]
pub struct ArenaTransitionParties {
    player_party: Entity,
    enemy_party: Entity,
}

fn begin_combat(
    mut commands: Commands,
    mut q_begin: Query<(Entity, &Transform, &mut BeginCombat, &DespawnTimer)>,
    mut state: ResMut<State<GameState>>,
    mut arena_state: ResMut<State<ArenaState>>,
    time: Res<Time>,
    mut egui: ResMut<EguiContext>,
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform, &OrthographicProjection)>,
) {
    for (entity, transform, mut begin, timer) in q_begin.iter_mut() {

        if let Ok((cam, global, proj)) = q_camera.get_single() {
            let window = windows.get_primary().unwrap();
            let pos = transform.translation + Vec3::new(0.0, 1.0, 0.0) * TILE_SIZE as f32;
            if let Some(mut pos) = cam.world_to_screen(&windows, global, pos) {
                pos.y = window.height() as f32 - pos.y;
            }
        }
        if timer.finished() {
            println!("Combat is beginning!");
            state.set(GameState::LoadArena).unwrap();
            commands.spawn().insert(ArenaCombat {
                player_party: begin.player_party,
                enemy_party: begin.enemy_party,
            });
        }
    }
}
