use rltk::{Rltk, GameState, RGB};
use specs::prelude::*;

mod components;
mod map;
mod player;
mod rect;

pub use components::*;
pub use map::*;
use player::*;
pub use rect::*;

pub struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        self.ecs.maintain(); // Tells Specs to apply any changes that are queued up.
    }
}

impl GameState for State {
    fn tick(&mut self, context: &mut Rltk) {
        context.cls(); // CLS: Clear the Screen.

        // Player Input
        player_input(self, context);

        // Here the ECS is calling out to our functions and components.
        self.run_systems(); // Within run_systems(...)

        // Render the Map
        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, context);

        // Here we're calling into the ECS to perform the Rendering
        let positions = self.ecs.read_storage::<Position>();
        let renderers = self.ecs.read_storage::<Renderer>();
        for (position, renderer) in (&positions, &renderers).join() {
            context.set(position.x, position.y, renderer.fg, renderer.bg, renderer.glyph)
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
        ecs: World::new()
    };

    // Register Components with ECS.
    game_state.ecs.register::<Position>();
    game_state.ecs.register::<Renderer>();
    game_state.ecs.register::<Player>();

    // Generate the Map
    let (rooms, map) = new_map_rooms_and_corridors();
    // Add resources to the ECS.
    game_state.ecs.insert(map);

    // Create Player
    let (player_x, player_y) = rooms[0].center();
    game_state.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderer {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();


    // Run the main game loop.
    rltk::main_loop(context, game_state)
}
