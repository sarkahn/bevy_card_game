use bevy::prelude::*;
use rand::{thread_rng, prelude::SliceRandom, Rng};

use crate::{ldtk_loader::LdtkMap, GENERATE_PARTY_SYSTEM, TILE_SIZE};

pub struct PartyPlugin;

impl Plugin for PartyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system(generate.label(GENERATE_PARTY_SYSTEM));

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
    map_sprite: Entity,
    arena_sprite: Entity,
}

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

        // Try load
        for name in &gen.names {
            if ldtk.get(name).is_none() {
                continue;
            }
        }

        let mut units = Vec::new();

        let icon = rng.gen_range(0..gen.count);

        for i in 0..gen.count {
            let to_spawn = gen.names.choose(&mut rng).unwrap();

            if let Some(ldtk) = ldtk.get(to_spawn) {

                //map_sprite.transform.translation = gen.pos;
                let mut map_sprite = get_tagged_sprite(ldtk, "map_sprite", 64.0);
                if i == icon {
                    map_sprite.visibility.is_visible = true;
                }
                let map_sprite = commands.spawn().insert_bundle(map_sprite).id();

                let arena_sprite = get_tagged_sprite(ldtk, "arena_sprite", 256.0);
                let arena_sprite = commands.spawn().insert_bundle(arena_sprite).id();
        
                let unit = PartyUnit {
                    map_sprite: map_sprite.clone(),
                    arena_sprite: arena_sprite.clone()
                };

                let unit = commands.spawn()
                .insert(unit)
                .insert(Transform::default())
                .insert(GlobalTransform::default())
                .add_child(map_sprite)
                .add_child(arena_sprite)
                .id();

                units.push(unit);
            } else {
                warn!("Tries to load ldtk {} while generating party, but it failed!", to_spawn);
            }
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

fn get_tagged_sprite(
    ldtk: &LdtkMap,
    tag: &str,
    size: f32,
) -> SpriteSheetBundle {
    let map_sprite = ldtk.get_tagged(tag).next().unwrap_or_else(||
        panic!("Error spawning unit {}, missing {} tag", ldtk.name(), tag)
    );
    let tileset = map_sprite.tileset_id().unwrap_or_else(||
        panic!("Error spawning unit {} {}, missing tileset id. Is a tilemap attached
        to the entity?", ldtk.name(), tag)
    );
    let tileset = ldtk.tileset_from_id(tileset).unwrap_or_else(||
        panic!("Error spawning unit {} {}, invalid tileset id. Is a tilemap attached
        to the entity?", ldtk.name(), tag)
    );
    let tile_id = map_sprite.tile_id().unwrap_or_else(||
        panic!("Error spawning unit {} {}, invalid tile id", ldtk.name(), tag)
    );
    get_sprite(tile_id as usize, tileset.atlas().clone(), Vec2::splat(size))
}

fn get_sprite(
    index: usize,
    atlas: Handle<TextureAtlas>,
    size: Vec2,
) -> SpriteSheetBundle {
    let sprite = TextureAtlasSprite {
        index,
        custom_size: Some(size),
        ..Default::default()
    };
    let xyz = Vec3::new(0.5, 0.5, 0.0) * TILE_SIZE as f32;
    SpriteSheetBundle {
        sprite,
        texture_atlas: atlas,
        transform: Transform::from_translation(xyz),
        visibility: Visibility { is_visible: false },
        ..Default::default()
    }
}