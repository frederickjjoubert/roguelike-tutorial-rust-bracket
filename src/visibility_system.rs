use specs::prelude::*;
use super::{Viewshed, Position, Map};
use rltk::{field_of_view, Point};
use crate::Player;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Viewshed>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut map, player, position, mut viewshed) = data;
        for (entity, position, viewshed) in (&entities, &position, &mut viewshed).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                let point = Point::new(position.x, position.y);
                viewshed.visible_tiles = field_of_view(point, viewshed.range, &*map);
                viewshed.visible_tiles.retain(|pos| pos.x >= 0 && pos.x < map.width && pos.y >= 0 && pos.y < map.height);

                // If this is the player, reveal what they can see.
                let player: Option<&Player> = player.get(entity);
                if let Some(_player) = player {
                    for tile in map.visible_tiles.iter_mut() {
                        *tile = false
                    };
                    for tile in viewshed.visible_tiles.iter() {
                        let index = map.xy_idx(tile.x, tile.y);
                        map.revealed_tiles[index] = true;
                        map.visible_tiles[index] = true;
                    }
                }
            }
        }
    }
}