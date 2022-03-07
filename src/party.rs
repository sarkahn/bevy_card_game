use bevy::prelude::*;
use rand::{thread_rng, prelude::SliceRandom, Rng};

use crate::{ldtk_loader::LdtkMap, GENERATE_PARTY_SYSTEM};

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
    //arena_sprite: Entity,
}

impl PartyUnit {
    /// Get the party unit's map sprite.
    pub fn map_sprite(&self) -> Entity {
        self.map_sprite
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
                let map_sprite = ldtk.get_tagged("map_sprite").next().unwrap_or_else(||
                    panic!("Error spawning unit {}, missing 'map_sprite' tag", ldtk.name())
                );
                let tileset = map_sprite.tileset_id().unwrap_or_else(||
                    panic!("Error spawning unit {} map sprite, missing tileset id. Is a tilemap attached
                    to the entity?", ldtk.name())
                );
                let tileset = ldtk.tileset_from_id(tileset).unwrap_or_else(||
                    panic!("Error spawning unit {} map sprite, invalid tileset id. Is a tilemap attached
                    to the entity?", ldtk.name())
                );
                let tile_id = map_sprite.tile_id().unwrap_or_else(||
                    panic!("Error spawning unit {} map sprite, invalid tile id", ldtk.name())
                );
                let mut map_sprite = get_sprite(tile_id as usize, tileset.atlas().clone());
                if i == icon {
                    map_sprite.visibility.is_visible = true;
                }
                //map_sprite.transform.translation = gen.pos;

                let map_sprite = commands.spawn().insert_bundle(map_sprite).id();

                let unit = PartyUnit {
                    map_sprite: map_sprite.clone()
                };

                let unit = commands.spawn()
                .insert(unit)
                .insert(Transform::default())
                .insert(GlobalTransform::default())
                .add_child(map_sprite).id();

                units.push(unit);
            } else {
                warn!("Tries to load ldtk {} while generating party, but it failed!", to_spawn);
            }
        }
        
        //println!("Spawning party, units: {:?}", units);
        commands.entity(entity).insert(Party).push_children(&units)
        .insert(Transform::from_translation(gen.pos))
        .insert(GlobalTransform::default())
        .remove::<GenerateParty>();
    }
}

fn get_sprite(
    index: usize,
    atlas: Handle<TextureAtlas>,
) -> SpriteSheetBundle {
    let sprite = TextureAtlasSprite {
        index,
        ..Default::default()
    };
    SpriteSheetBundle {
        sprite,
        texture_atlas: atlas,
        visibility: Visibility { is_visible: false },
        ..Default::default()
    }
}