
fn on_spawn(
    mut commands: Commands, 
    ldtk: Res<Assets<LdtkMap>>,
    config: Res<Assets<ConfigAsset>>,
    atlas: Res<CardsAtlas>,
    q_spawn: Query<(Entity,&SpawnCard)>,
    asset_server: Res<AssetServer>,
) {
    if let Some(config) = config.get(SETTINGS_PATH) {
        if let Some(ldtk) = ldtk.get(&config.settings.arena_file) {
            for (entity,spawn) in q_spawn.iter() {
                commands.entity(entity).despawn();

                let data_layer = ldtk.layer_from_name("Card_Data").unwrap_or_else(||
                    panic!("Couldn't find 'Card_Data' in ldtk file {}", ldtk.name()));
                
                let entities = data_layer.as_entities();
             
                let font: Handle<Font> = asset_server.load("fonts/DejaVuSerif.ttf");
                //spawn_card(&mut commands, entities.unwrap(), atlas.0.clone(), spawn.element, font);
            }
        }
    }
}
