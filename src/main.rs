use rltk::{Rltk, GameState, RGB, FontCharType, Point};
use specs::prelude::*;

mod components;
mod damage_system;
mod game_log;
mod gui;
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
pub use map::*;
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use monster_ai_system::MonsterAI;
use player::*;
pub use rect::Rect;
pub use visibility_system::VisibilitySystem;

pub struct State {
    pub ecs: World,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
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
        self.ecs.maintain(); // Tells Specs to apply any changes that are queued up.
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        context.cls(); // CLS: Clear the Screen.

        // Get RunState resource
        let current_run_state = *self.ecs.fetch::<RunState>();
        let new_run_state: RunState;

        // Match on RunState
        match current_run_state {
            RunState::PreRun => {
                self.run_systems();
                new_run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                // Player Input
                new_run_state = player_input(self, context);
            }
            RunState::PlayerTurn => {
                // Here the ECS is calling out to our functions and components.
                self.run_systems(); // Within run_systems(...)
                new_run_state = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                new_run_state = RunState::AwaitingInput;
            }
        }

        // Update RunState resource.
        {
            let mut run_state_writer = self.ecs.write_resource::<RunState>();
            *run_state_writer = new_run_state;
        }

        // Clean up the dead.
        damage_system::delete_the_dead(&mut self.ecs);

        // Render Loop
        // Render the Map
        draw_map(&self.ecs, context);
        // Render Entities: Here we're calling into the ECS to perform the Rendering
        let positions = self.ecs.read_storage::<Position>();
        let renderers = self.ecs.read_storage::<Renderer>();
        let map = self.ecs.fetch::<Map>();
        for (position, renderer) in (&positions, &renderers).join() {
            let index = map.xy_idx(position.x, position.y);
            if map.visible_tiles[index] {
                context.set(position.x, position.y, renderer.fg, renderer.bg, renderer.glyph)
            }
        }

        // ^ It can be a tough judgment call on which to use.
        // If your system just needs data from the ECS,
        // then a system is the right place to put it.
        // If it also needs access to other parts of your program,
        // it is probably better implemented on the outside - calling in.

        // Draw UI
        gui::draw_ui(&self.ecs, context);
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike")
        .build()?;
    context.with_post_scanlines(true);
    let mut game_state = State {
        ecs: World::new(),
    };

    // Register Components with ECS.
    game_state.ecs.register::<BlocksTile>();
    game_state.ecs.register::<CombatStats>();
    game_state.ecs.register::<Monster>();
    game_state.ecs.register::<Name>();
    game_state.ecs.register::<Player>();
    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Renderer>();
    game_state.ecs.register::<SufferDamage>();
    game_state.ecs.register::<Viewshed>();
    game_state.ecs.register::<WantsToMelee>();

    // Generate the Map
    let map = Map::new_map_rooms_and_corridors();

    // Get Player Start Position
    let (player_x, player_y) = map.rooms[0].center();

    // Create Player
    let player_entity = game_state.ecs
        .create_entity()
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(Name {
            name: "Player".to_string()
        })
        .with(Player {})
        .with(Position { x: player_x, y: player_y })
        .with(Renderer {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .build();


    // Create Monsters
    let mut rng = rltk::RandomNumberGenerator::new();

    for (index, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();
        let name: String;
        let glyph: FontCharType;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                name = "Goblin".to_string();
                glyph = rltk::to_cp437('g')
            }
            _ => {
                name = "Orc".to_string();
                glyph = rltk::to_cp437('o')
            }
        }
        game_state.ecs
            .create_entity()
            .with(BlocksTile {})
            .with(CombatStats {
                max_hp: 16,
                hp: 16,
                defense: 1,
                power: 4,
            })
            .with(Monster {})
            .with(Name {
                name: format!("{} #{}", name, index)
            })
            .with(Position { x, y })
            .with(Renderer {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .build();
    }

    // Add resources to the ECS. (Kinda like global variables?)
    game_state.ecs.insert(game_log::GameLog { entries: vec!["You find yourself in a dark room with no recollection of who you are.".to_string()] });
    game_state.ecs.insert(map);
    game_state.ecs.insert(player_entity);
    game_state.ecs.insert(Point::new(player_x, player_y));
    game_state.ecs.insert(rltk::RandomNumberGenerator::new());
    game_state.ecs.insert(RunState::PreRun);

    // Run the main game loop.
    rltk::main_loop(context, game_state)
}
