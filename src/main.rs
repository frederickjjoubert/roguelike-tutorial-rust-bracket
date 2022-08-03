use rltk::{Rltk, GameState, Point};
use specs::prelude::*;

mod components;
mod damage_system;
mod game_log;
mod gui;
mod inventory_system;
mod map;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod player;
mod rect;
mod spawner;
mod visibility_system;

pub use components::*;
use damage_system::DamageSystem;
pub use game_log::GameLog;
pub use map::*;
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use monster_ai_system::MonsterAI;
use player::*;
pub use rect::Rect;
pub use visibility_system::VisibilitySystem;
use crate::inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem};

pub struct State {
    pub ecs: World,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
}

impl State {
    fn run_systems(&mut self) {
        let mut visibility_system = VisibilitySystem {};
        visibility_system.run_now(&self.ecs);
        let mut monster_ai_system = MonsterAI {};
        monster_ai_system.run_now(&self.ecs);
        let mut map_indexing_system = MapIndexingSystem {};
        map_indexing_system.run_now(&self.ecs);
        let mut melee_combat_system = MeleeCombatSystem {};
        melee_combat_system.run_now(&self.ecs);
        let mut damage_system = DamageSystem {};
        damage_system.run_now(&self.ecs);
        let mut item_collection_system = ItemCollectionSystem {};
        item_collection_system.run_now(&self.ecs);
        let mut item_use_system = ItemUseSystem {};
        item_use_system.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);
        self.ecs.maintain(); // Tells Specs to apply any changes that are queued up.
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        context.cls(); // Clear the Screen.

        // === Render Loop ===

        // Render the Map
        draw_map(&self.ecs, context);

        {
            let map = self.ecs.fetch::<Map>();
            // Render Entities: Here we're calling into the ECS to perform the Rendering
            let positions = self.ecs.read_storage::<Position>();
            let renderers = self.ecs.read_storage::<Renderer>();
            let mut render_data = (&positions, &renderers)
                .join()
                .collect::<Vec<_>>();
            render_data.sort_by(
                |&a, &b|
                    b.1.render_order.cmp(&a.1.render_order)
            );
            for (position, renderer) in render_data.iter() {
                let index = map.xy_idx(position.x, position.y);
                if map.visible_tiles[index] {
                    context.set(position.x, position.y, renderer.fg, renderer.bg, renderer.glyph)
                }
            }

            // Draw UI
            gui::draw_ui(&self.ecs, context);
        }

        // === Input Loop ===
        // Get RunState resource
        let current_run_state = *self.ecs.fetch::<RunState>();
        let mut new_run_state = current_run_state;

        // Match on RunState
        match current_run_state {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                // Player Input
                new_run_state = player_input(self, context);
            }
            RunState::PlayerTurn => {
                // Here the ECS is calling out to our functions and components.
                self.run_systems(); // Within run_systems(...)
                self.ecs.maintain();
                new_run_state = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, context);
                let item_menu_result = result.0;

                match item_menu_result {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {} // Do Nothing
                    gui::ItemMenuResult::Selected => {
                        // We're unwrapping here because if we have ItemMenuResult::Selected we know there must be an item from show_inventory(...)
                        let item_entity = result.1.unwrap();
                        let mut wants_to_use_item_components = self.ecs.write_storage::<WantsToUseItem>();
                        let player_entity = self.ecs.fetch::<Entity>();
                        wants_to_use_item_components.insert(
                            *player_entity,
                            WantsToUseItem {
                                item: item_entity,
                                target: None,
                            },
                        ).expect("Unable to insert WantsToUseItem component.");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, context);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_run_state = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem { item: item_entity }).expect("Unable to insert intent");
                        new_run_state = RunState::PlayerTurn;
                    }
                }
            }
        }

        // Update RunState resource.
        {
            let mut run_state_writer = self.ecs.write_resource::<RunState>();
            *run_state_writer = new_run_state;
        }

        // Clean up the dead.
        damage_system::delete_the_dead(&mut self.ecs);
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike")
        .build()?;
    context.with_post_scanlines(true); // Post Processing Effect.
    let mut game_state = State {
        ecs: World::new(),
    };

    // Register Components with ECS.
    game_state.ecs.register::<BlocksTile>();
    game_state.ecs.register::<CombatStats>();
    game_state.ecs.register::<Consumable>();
    game_state.ecs.register::<InBackpack>();
    game_state.ecs.register::<Item>();
    game_state.ecs.register::<Monster>();
    game_state.ecs.register::<Name>();
    game_state.ecs.register::<Player>();
    game_state.ecs.register::<Position>();
    game_state.ecs.register::<ProvidesHealing>();
    game_state.ecs.register::<Renderer>();
    game_state.ecs.register::<SufferDamage>();
    game_state.ecs.register::<Viewshed>();
    game_state.ecs.register::<WantsToDrinkPotion>();
    game_state.ecs.register::<WantsToDropItem>();
    game_state.ecs.register::<WantsToMelee>();
    game_state.ecs.register::<WantsToPickupItem>();
    game_state.ecs.register::<WantsToUseItem>();


    // Generate the Map
    let map = Map::new_map_rooms_and_corridors();

    // Get Player Start Position
    let (player_x, player_y) = map.rooms[0].center();

    // Create Player
    let player_entity = spawner::spawn_player(&mut game_state.ecs, player_x, player_y);

    // Insert the RNG as an ECS Resource
    game_state.ecs.insert(rltk::RandomNumberGenerator::new());

    // Create Monsters
    for room in map.rooms.iter().skip(1) {
        spawner::fill_room(&mut game_state.ecs, room);
    };

    // Add resources to the ECS. (Kinda like global variables?)
    game_state.ecs.insert(game_log::GameLog {
        entries: vec!["You find yourself in a dark room with no recollection of who you are.".to_string()]
    });
    game_state.ecs.insert(map);
    game_state.ecs.insert(player_entity);
    game_state.ecs.insert(Point::new(player_x, player_y));
    game_state.ecs.insert(RunState::PreRun);

    // Run the main game loop.
    rltk::main_loop(context, game_state)
}
