use bevy::{prelude::*, utils::label::DynEq};
use rand::{prelude::SliceRandom, thread_rng, Rng};

use crate::{
    config::ConfigAsset, ldtk_loader::*, make_sprite, make_sprite_atlas_sized,
    make_sprite_image_sized, 
    //prefab::ChangeSprite, 
    unit::{Element, Player, Enemy}, AtlasHandles, GameState,
    //LoadCardPrefab, SpawnPrefabOld, 
    SETTINGS_PATH, TILE_SIZE, animation::{Animator, AnimationCommand}, make_spritesheet_bundle, party::{PartyUnit, GenerateParty, Party, PartyUnitSprite}, GENERATE_PARTY_SYSTEM,
};

use super::{cards::{CardLabel, CardLabelType, CardsAtlas, SpawnCard}, TakingATurn, ArenaCombat};

pub struct ArenaLoadPlugin;

impl Plugin for ArenaLoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CardsAtlas>()
            .add_system_set(SystemSet::on_enter(GameState::LoadArena)
                .with_system(load_data))
            .add_system_set(SystemSet::on_update(GameState::LoadArena)
                .with_system(setup.after(GENERATE_PARTY_SYSTEM))

        )
        ;
    }
}

pub struct LdtkHandles(Vec<Handle<LdtkMap>>);


/*
    let names = config.settings.player_units.iter().map(|s|s.to_string()).collect();
    //println!("Spawning gen...");
    commands.spawn().insert(
        GenerateParty::new(4, names, pos),
    ).insert(Player);
*/

fn load_data(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<Assets<ConfigAsset>>,
    q_parties: Query<&ArenaCombat>,
) {
    let config = config.get(SETTINGS_PATH).unwrap();
    let handle: Handle<LdtkMap> = asset_server.load(&config.settings.arena_file);
    let handles = vec![handle];
    commands.insert_resource(LdtkHandles(handles));

    if q_parties.is_empty() {
        warn!("No combat parties found when loading arena scene, generating parties for debug purposes.");

        let player_units = config.settings.player_units.to_owned();
        let enemy_units = config.settings.enemy_units.to_owned();
        let player_units = commands.spawn().insert(GenerateParty::new(4, player_units, Vec3::ZERO))
        .insert(Player);
        commands.spawn().insert(GenerateParty::new(4, enemy_units, Vec3::ZERO))
        .insert(Enemy);

    }
}

fn setup(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    mut card_atlas: ResMut<CardsAtlas>,
    ldtk: Res<Assets<LdtkMap>>,
    config: Res<Assets<ConfigAsset>>,
    q_combat: Query<&ArenaCombat>,
    q_units: Query<&Children>,
    q_unit: Query<&PartyUnit>,
    q_parties: Query<Entity, With<Party>>,
    q_player: Query<Entity, With<Player>>,
    q_enemy: Query<Entity, With<Enemy>>,
    mut q_sprite: Query<(Entity, &mut Visibility, &mut Transform, &mut GlobalTransform), Without<Camera>>,
    q_cam: Query<&Transform, With<Camera>>,
) {
    let config = config.get(SETTINGS_PATH).unwrap();
    if let Some(ldtk) = ldtk.get(&config.settings.arena_file) {
        if q_combat.is_empty() {
            if q_parties.is_empty() {
                // Wait for party to generate
                return;
            }
            let player_party = q_player.single();
            let enemy_party = q_enemy.single();
            //println!("Found debug parties, intiiating combat");
            commands.spawn().insert(ArenaCombat {
                player_party,
                enemy_party,
            });
            // Let commands execute
            return;
        }

        if let Some(bg) = &ldtk.background() {
            let mut pos = q_cam.single().translation;
            pos.z = 10.0;
            commands.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(ldtk.size_px().as_vec2()),
                    ..Default::default()
                },
                texture: bg.image.clone(),
                transform: Transform::from_translation(pos),
                ..Default::default()
            });
        }


        if let Ok(combat) = q_combat.get_single() {

            let spawns: Vec<_> = ldtk.get_tagged("spawn_point").collect();
            let player_spawns = spawns.iter().filter(|e|e.tags().has("player") && !e.tags().has("card"));
    
            let actions = player_spawns.clone().map(|e|try_get_actions(e.fields(), "attackactions"));
    
            let player_units = q_units.get(combat.player_party).unwrap();
            for (i,(spawn,unit)) in player_spawns.zip(player_units.iter()).enumerate() {
                let unit = q_unit.get(*unit).unwrap();
                let (sprite_entity, mut visibility, mut transform, global) = q_sprite.get_mut(unit.arena_sprite()).unwrap();
    
                let p = spawn.xy().as_vec2().extend(20.0 + i as f32);
                let local_pos = global.compute_matrix().inverse().transform_point3(p);
    
                transform.translation = local_pos;
                visibility.is_visible = true;
            }
    
            let enemy_spawns = spawns.iter().filter(|e|e.tags().has("enemy") && !e.tags().has("card"));
    
            let actions = enemy_spawns.clone().map(|e|try_get_actions(e.fields(), "attackactions"));
    
            let enemy_units = q_units.get(combat.enemy_party).unwrap();
            for (i,(spawn,unit)) in enemy_spawns.rev().zip(enemy_units.iter()).enumerate() {
                let unit = q_unit.get(*unit).unwrap();
                let (sprite_entity, mut visibility, mut transform, global) = q_sprite.get_mut(unit.arena_sprite()).unwrap();
    
                let p = spawn.xy().as_vec2().extend(15.0 + i as f32);
                let local_pos = global.compute_matrix().inverse().transform_point3(p);
    
                transform.translation = local_pos;
                visibility.is_visible = true;
            }

            commands.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(TILE_SIZE as f32)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(50.0,50.0,30.0),
                ..Default::default()
            });
    
        }

        // let actions = actions.take_while(|a|a.is_some()).next().unwrap().unwrap();
        // println!("Found {} unit actions: {:?}", actions.len(), actions);

        // for (i,spawn) in player_spawns.enumerate() {

        //     let mut animator = Animator::default();
        //     if let Some(actions) = try_get_actions(spawn.fields(), "attackactions") {
        //         //println!("Found attack acionts for {}", spawn.name());
        //         //animator.add_commands(actions);
        //     }
        //     animator.push_cmds_back(actions.clone());
        //     animator.push_cmds_front([
        //         AnimationCommand::Play("idle".to_string()),
        //         AnimationCommand::Wait(1.5 * i as f32)]
        //     );
 

        //     // let xy = spawn.xy();
        //     // commands.spawn().insert(SpawnPrefabOld {
        //     //     path: "units_wizard.ldtk".to_string(),
        //     //     xy: xy,
        //     //     depth: 10 + i as i32,
        //     //     ..Default::default()
        //     // })
        //     // .insert(animator)
        //     // ;
        // }

        // let slimes = spawns.iter().filter(|e|e.tags().has("enemy"));


        // for (i,spawn) in slimes.rev().enumerate() {
        //     let xy = spawn.xy();
        //     // commands.spawn().insert(SpawnPrefabOld {
        //     //     path: "units_slime.ldtk".to_string(),
        //     //     xy: xy,
        //     //     depth: 10 + i as i32,
        //     //     ..Default::default()
        //     // });
        // }

        // let cards: Vec<_> = cards_pfb
        //     .get_tagged("card")
        //     .filter(|card| {
        //         if let Some(rarity) = card.fields().try_get_str("rarity") {
        //             return rarity == "common" || rarity == "uncommon";
        //         }
        //         false
        //     })
        //     .collect();

        // let spawns = spawns.iter().filter(|e| e.tags().has("card"));

        // let mut rng = thread_rng();

        // for (i,spawn) in spawns.enumerate() {
        //     let card = cards.choose(&mut rng).unwrap();

        //     let texture = card.fields().try_get_str("texture").unwrap_or_else(|| {
        //         panic!(
        //             "Error loading card {}, couldn't parse texture field ",
        //             spawn.name()
        //         )
        //     });

        //     let tileset = cards_pfb.tileset_from_path(texture).unwrap_or_else(|| {
        //         panic!(
        //             "Error loading tileset, {} wasn't found in the ldtk file",
        //             texture
        //         )
        //     });

        //     let tile_id = card
        //         .fields()
        //         .try_get_i32("sprite_index")
        //         .unwrap_or_else(|| panic!("Error loading 'tile_id' from {}", spawn.name()));

        //     let atlas = get_atlas(&mut atlases, &mut atlas_handles, tileset);

        //     let xy = spawn.xy();
        //     //println!("Spawning card {}", card.name());
        //     // commands.spawn().insert(LoadCardPrefab {
        //     //     path: "units_BattleCardPremade.ldtk".to_string(),
        //     //     xy,
        //     //     depth: 5 + i as i32,
        //     //     atlas,
        //     //     tile_id,
        //     //     size: card.size(),
        //     // });
        // }

        state.set(GameState::Arena).unwrap();
    }
}


fn get_actions<'a>(
    fields: &'a Fields,
    name: &'a str
) -> impl Iterator<Item=AnimationCommand> + 'a {
    let vals = fields.field("AttackActions").unwrap().as_array().unwrap();
    let vals = vals.iter().map(|v|v.as_str().unwrap().to_lowercase());
    vals.map(|v|ron::de::from_str(&v).unwrap())
}

fn try_get_actions<'a>(
    fields: &'a Fields,
    name: &'a str,
) -> Option<Vec<AnimationCommand>> {
    if let Some(field) = fields.field(name) {
        //println!("Found field {:?}", field);
        if let Some(arr) = field.as_array() {
            //println!("Found arr {:?}", arr);
            let strings = arr.iter().map(|v|v.as_str().unwrap());
            let commands = strings.map(|s|ron::de::from_str::<AnimationCommand>(s));
            return Some(commands.take_while(|c|c.is_ok()).map(|c|c.unwrap()).collect());
        } 
    }
    None
} 

fn play_turn(

) {
    
}

fn get_atlas(
    atlases: &mut Assets<TextureAtlas>,
    atlas_handles: &mut AtlasHandles,
    tileset: &MapTileset,
) -> Handle<TextureAtlas> {
    let name = &tileset.name;
    match atlas_handles.0.get(name) {
        Some(atlas) => atlas.clone(),
        None => {
            let atlas = TextureAtlas::from_grid(
                tileset.image.clone(),
                IVec2::splat(tileset.tile_size).as_vec2(),
                tileset.tile_count.x as usize,
                tileset.tile_count.y as usize,
            );
            let handle = atlases.add(atlas);
            atlas_handles.0.insert(name.to_string(), handle.clone());
            handle
        }
    }
}
