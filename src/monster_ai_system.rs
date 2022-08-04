use specs::prelude::*;
use super::{
    Confusion,
    Map,
    Monster,
    Point,
    Position,
    RunState,
    Viewshed,
    WantsToMelee,
};


pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)] // To tell the linter that we really did mean to use quite so much in one type!
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, RunState>,
        WriteExpect<'a, Map>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, WantsToMelee>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            player_entity,
            player_position,
            run_state,
            mut map,
            monsters,
            mut confusion,
            mut positions,
            mut viewsheds,
            mut wants_to_melee,
        ) = data;

        // Only run the MonsterAI System if the RunState is the MonstersTurn
        if *run_state != RunState::MonsterTurn { return; }

        for (entity, _monster, mut monster_position, mut monster_viewshed)
        in (&entities, &monsters, &mut positions, &mut viewsheds).join() {
            let mut can_act = true;

            let is_confused = confusion.get_mut(entity);
            if let Some(is_confused) = is_confused {
                is_confused.turns -= 1;
                if is_confused.turns < 1 {
                    confusion.remove(entity);
                }
                can_act = false;
            }

            if can_act {
                let distance_to_player = rltk::DistanceAlg::Pythagoras.distance2d(
                    Point::new(monster_position.x, monster_position.y),
                    *player_position,
                );
                if distance_to_player < 1.5 {
                    wants_to_melee.insert(entity, WantsToMelee { target: *player_entity }).expect("Unable to insert WantsToMelee component.");
                    return;
                } else if monster_viewshed.visible_tiles.contains(&*player_position) {
                    let path_to_player = rltk::a_star_search(
                        map.xy_idx(monster_position.x, monster_position.y),
                        map.xy_idx(player_position.x, player_position.y),
                        &mut *map);
                    if path_to_player.success && path_to_player.steps.len() > 1 {
                        // Unblock current position.
                        let index = map.xy_idx(monster_position.x, monster_position.y);
                        map.blocked_tiles[index] = false;
                        // Move along path to the next position.
                        monster_position.x = path_to_player.steps[1] as i32 % map.width;
                        monster_position.y = path_to_player.steps[1] as i32 / map.width;
                        // Block new position.
                        let index = map.xy_idx(monster_position.x, monster_position.y);
                        map.blocked_tiles[index] = true;
                        // Viewshed needs to update now.
                        monster_viewshed.dirty = true;
                    }
                }
            }
        }
    }
}