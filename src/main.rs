use rltk::{Rltk, GameState, RGB, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{min, max};
use specs_derive::Component;

// Components
#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

// An empty Component is called a Tag Component
#[derive(Component)]
struct LeftMover {}

#[derive(Component, Debug)]
struct Player {}

// Systems
struct LeftWalker {}

// impl<'a> System<'a> for LeftWalker means we are implementing Specs' System trait
// for our LeftWalker structure. The 'a are lifetime specifiers: the system is saying
// that the components it uses must exist long enough for the system to run.
impl<'a> System<'a> for LeftWalker {
    // This system needs Read Access from LeftMover, and Write Access to Position
    type SystemData = (
        ReadStorage<'a, LeftMover>,
        WriteStorage<'a, Position>
    );

    // fn run is the actual trait implementation, required by the impl System.
    // It takes itself, and the SystemData we defined.
    fn run(&mut self, (lefty, mut position): Self::SystemData) {
        for (_, position) in (&lefty, &mut position).join() {
            position.x -= 1;
            if position.x < 0 {
                position.x = 79;
            }
        }
    }
}

// Other
struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut left_walker = LeftWalker {}; // Create a new (changeable) instance of the LeftWalker system.
        left_walker.run_now(&self.ecs); // Run the System.
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

        // Here we're calling into the ECS to perform the Rendering
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (position, renderable) in (&positions, &renderables).join() {
            context.set(position.x, position.y, renderable.fg, renderable.bg, renderable.glyph)
        }

        // ^ It can be a tough judgment call on which to use.
        // If your system just needs data from the ECS,
        // then a system is the right place to put it.
        // If it also needs access to other parts of your program,
        // it is probably better implemented on the outside - calling in.
    }
}

fn player_input(game_state: &mut State, context: &mut Rltk) {
    match context.key {
        None => {} // No Input, Do Nothing.
        Some(key) => {
            match key {
                VirtualKeyCode::Left => {
                    try_move_player(-1, 0, &mut game_state.ecs);
                }
                VirtualKeyCode::Right => {
                    try_move_player(1, 0, &mut game_state.ecs);
                }
                VirtualKeyCode::Up => {
                    try_move_player(0, -1, &mut game_state.ecs);
                }
                VirtualKeyCode::Down => {
                    try_move_player(0, 1, &mut game_state.ecs);
                }
                _ => {} // Anything else, Do Nothing.
            }
        }
    }
}

fn try_move_player(dx: i32, dy: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();

    for (_player, position) in (&mut players, &mut positions).join() {
        // Check you haven't left the screen.
        position.x = min(79, max(0, position.x + dx));
        position.y = min(49, max(0, position.y + dy));
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
    game_state.ecs.register::<Renderable>();
    game_state.ecs.register::<LeftMover>();
    game_state.ecs.register::<Player>();
    // Create Entities
    // Create Player
    game_state.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    // Create NPCs
    for i in 0..10 {
        game_state.ecs
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(Renderable {
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(LeftMover {})
            .build();
    }
    // Run the main game loop.
    rltk::main_loop(context, game_state)
}
