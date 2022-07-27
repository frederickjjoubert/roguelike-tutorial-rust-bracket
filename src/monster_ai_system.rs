use specs::prelude::*;
use super::{Map, Monster, Name, Position, Viewshed};
use rltk::{field_of_view, Point, console};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        ReadExpect<'a, Point>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Viewshed>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (monsters, names, player_position, positions, viewsheds) = data;
        for (monster, name, position, viewshed) in (&monsters, &names, &positions, &viewsheds).join() {
            if (viewshed.visible_tiles.contains(&*player_position)) {
                // console::log is a rltk helper, not std rust.
                console::log(&format!("The {} sees you and growls!", { &name.name }));
            }
        }
    }
}