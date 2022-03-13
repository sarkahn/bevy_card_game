use bevy::prelude::*;
use rand::{thread_rng, prelude::SliceRandom, Rng};

use crate::{ldtk_loader::LdtkMap, GENERATE_PARTY_SYSTEM, TILE_SIZE, BuildPrefab};

pub struct PartyPlugin;

impl Plugin for PartyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system(generate.label(GENERATE_PARTY_SYSTEM))
        .add_system(show_map_sprite)
        ;

    }
}

#[derive(Component)]
pub struct GenerateParty {
    count: usize,
    names: Vec<String>,
    pos: Vec3,
}
impl GenerateParty {
    pub fn new(count: usize, names: Vec<String>, pos: Vec3) -> Self {
        Self {
            count,
            names,
            pos,
        }
    }
}

#[derive(Component)]
pub struct Party;

#[derive(Component)]
pub struct PartyUnit {
    pub map_sprite: Entity,
    pub arena_sprite: Entity,
}

#[derive(Component)]
pub struct ShowMapSprite;

#[derive(Component)]
pub struct ArenaSpriteVisibility(pub bool);

#[derive(Component)]
pub struct PartyUnitSprite;

impl PartyUnit {
    /// Get the party unit's map sprite.
    pub fn map_sprite(&self) -> Entity {
        self.map_sprite
    }

    /// Get the party unit's arena sprite.
    pub fn arena_sprite(&self) -> Entity {
        self.arena_sprite
    }
}

fn generate(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ldtk: Res<Assets<LdtkMap>>,
    mut q_gen: Query<(Entity, &mut GenerateParty)>,
) {
    let mut rng = thread_rng();
    for (entity, gen) in q_gen.iter_mut() {
        //println!("Generating party");

        if gen.names.iter().any(|n|ldtk.get(n).is_none()) {
            warn!("Error spawning party, unit prefabs were not loaded yet!");
            continue;
        }

        let mut units = Vec::new();

        let icon = rng.gen_range(0..gen.count);

        for i in 0..gen.count {
            let to_spawn = gen.names.choose(&mut rng).unwrap();

            let mut unit = commands.spawn();
            
            unit.insert(BuildPrefab {
                name: to_spawn.to_owned()
            });

            if i == icon {
                //println!("Inserting showmapsprite on {:?}", unit.id());
                unit.insert(ShowMapSprite);
            }

            let unit = unit.id();

            units.push(unit);
        }
        let pos = gen.pos;// + Vec3::new(0.5,0.5,0.0) * TILE_SIZE as f32;
        //println!("Spawning party, units: {:?}", units);
        commands.entity(entity)
        .insert(Party)
        .push_children(&units)
        .insert(Transform::from_translation(pos))
        .insert(GlobalTransform::default())
        .remove::<GenerateParty>();
    }
}

fn show_map_sprite(
    mut commands: Commands,
    q_unit: Query<(Entity,&PartyUnit), With<ShowMapSprite>>,
    mut q_vis: Query<&mut Visibility>,
) {
    for (entity, unit) in q_unit.iter() {
        //println!("Found showmapsprite unit!");
        if let Ok(mut vis) = q_vis.get_mut(unit.map_sprite) {
            vis.is_visible = true;
            //println!("Showing sprite");
            commands.entity(entity).remove::<ShowMapSprite>();
        }
    }
}

// fn get_tagged_sprite(
//     ldtk: &LdtkMap,
//     tag: &str,
//     size: f32,
// ) -> SpriteSheetBundle {
//     let map_sprite = ldtk.get_tagged(tag).next().unwrap_or_else(||
//         panic!("Error spawning unit {}, missing {} tag", ldtk.name(), tag)
//     );
//     let tileset = map_sprite.tileset_id().unwrap_or_else(||
//         panic!("Error spawning unit {} {}, missing tileset id. Is a tilemap attached
//         to the entity?", ldtk.name(), tag)
//     );
//     let tileset = ldtk.tileset_from_id(tileset).unwrap_or_else(||
//         panic!("Error spawning unit {} {}, invalid tileset id. Is a tilemap attached
//         to the entity?", ldtk.name(), tag)
//     );
//     let tile_id = map_sprite.tile_id().unwrap_or_else(||
//         panic!("Error spawning unit {} {}, invalid tile id", ldtk.name(), tag)
//     );
//     get_sprite(tile_id as usize, tileset.atlas().clone(), Vec2::splat(size))
// }

// fn get_sprite(
//     index: usize,
//     atlas: Handle<TextureAtlas>,
//     size: Vec2,
// ) -> SpriteSheetBundle {
//     let sprite = TextureAtlasSprite {
//         index,
//         custom_size: Some(size),
//         ..Default::default()
//     };
//     let xyz = Vec3::new(0.5, 0.5, 0.0) * TILE_SIZE as f32;
//     SpriteSheetBundle {
//         sprite,
//         texture_atlas: atlas,
//         transform: Transform::from_translation(xyz),
//         visibility: Visibility { is_visible: false },
//         ..Default::default()
//     }
// }