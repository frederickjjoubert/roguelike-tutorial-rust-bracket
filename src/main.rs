use rltk::{Rltk, GameState, RGB, FontCharType, Point};
use specs::prelude::*;

mod components;
mod map;
mod monster_ai_system;
mod player;
mod rect;
mod visibility_system;

pub use components::*;
pub use map::*;
pub use monster_ai_system::MonsterAI;
use player::*;
pub use rect::Rect;
pub use visibility_system::VisibilitySystem;

pub struct State {
    pub ecs: World,
    pub run_state: RunState,
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}

impl State {
    fn run_systems(&mut self) {
        let mut visibility_system = VisibilitySystem {};
        visibility_system.run_now(&self.ecs);
        let mut monster_ai_system = MonsterAI {};
        monster_ai_system.run_now(&self.ecs);
        self.ecs.maintain(); // Tells Specs to apply any changes that are queued up.
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        context.cls(); // CLS: Clear the Screen.

        match self.run_state {
            RunState::Paused => {
                // Player Input
                self.run_state = player_input(self, context);
            }
            RunState::Running => {
                // Here the ECS is calling out to our functions and components.
                self.run_systems(); // Within run_systems(...)
                self.run_state = RunState::Paused;
            }
        }

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
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike")
        .build()?;
    let mut game_state = State {
        ecs: World::new(),
        run_state: RunState::Running,
    };

    // Register Components with ECS.
    game_state.ecs.register::<Monster>();
    game_state.ecs.register::<Name>();
    game_state.ecs.register::<Player>();
    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Renderer>();
    game_state.ecs.register::<Viewshed>();

    // Generate the Map
    let map = Map::new_map_rooms_and_corridors();

    // Get Player Start Position
    let (player_x, player_y) = map.rooms[0].center();

    // Create Player
    game_state.ecs
        .create_entity()
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

    // Add resources to the ECS.
    game_state.ecs.insert(map);
    game_state.ecs.insert(Point::new(player_x, player_y));

    // Run the main game loop.
    rltk::main_loop(context, game_state)
}
