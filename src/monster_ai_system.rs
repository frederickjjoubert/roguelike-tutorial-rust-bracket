use specs::prelude::*;
use super::{Map, Monster, Name, Position, Viewshed};
use rltk::{Point, console};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)] // To tell the linter that we really did mean to use quite so much in one type!
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        ReadExpect<'a, Point>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Viewshed>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, monsters, names, player_position, mut positions, mut viewsheds) = data;
        for (_monster, name, mut monster_position, mut monster_viewshed) in (&monsters, &names, &mut positions, &mut viewsheds).join() {
            if monster_viewshed.visible_tiles.contains(&*player_position) {
                let distance_to_player = rltk::DistanceAlg::Pythagoras.distance2d(
                    Point::new(monster_position.x, monster_position.y),
                    *player_position,
                );
                if distance_to_player < 1.5 {
                    console::log(&format!("{} sees you and growls!", { &name.name })); // console::log is a rltk helper, not std rust.
                    return;
                }
                let path_to_player = rltk::a_star_search(
                    map.xy_idx(monster_position.x, monster_position.y),
                    map.xy_idx(player_position.x, player_position.y),
                    &mut *map);
                if path_to_player.success && path_to_player.steps.len() > 1 {
                    monster_position.x = path_to_player.steps[1] as i32 % map.width;
                    monster_position.y = path_to_player.steps[1] as i32 / map.width;
                    monster_viewshed.dirty = true;
                }
            }
        }
    }
}