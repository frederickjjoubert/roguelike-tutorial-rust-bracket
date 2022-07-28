use specs::prelude::*;
use super::{Map, Position, BlocksTile};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        ReadStorage<'a, BlocksTile>,
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (blocks_tiles, mut map, positions) = data;
        map.calculate_blocked_tiles();
        for (_, position) in (&blocks_tiles, &positions).join() {
            let index = map.xy_idx(position.x, position.y);
            map.blocked_tiles[index] = true;
        }
    }
}