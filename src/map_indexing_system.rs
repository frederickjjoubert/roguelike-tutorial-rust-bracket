use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (blocks_tiles, entities, mut map, positions) = data;

        map.calculate_blocked_tiles();
        map.clear_all_tiles_contents();

        for (entity, position) in (&entities, &positions).join() {
            let index = map.xy_idx(position.x, position.y);

            // Check if this Entity blocks the Tile.
            let blocker = blocks_tiles.get(entity);
            if let Some(_blocker) = blocker {
                map.blocked_tiles[index] = true;
            }

            // Push the Entity to the Tile Contents
            // Note: Entity is a Copy type,
            // so we don't need to clone it
            // (we want to avoid moving it out of the ECS!)
            map.tile_contents[index].push(entity);
        }
    }
}