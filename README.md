# Roguelike Tutorial by Bracket

This is my code following the open source roguelike tutorial series by [Herbert Wolverson](https://github.com/thebracket).

The tutorial page is [here](https://bfnightly.bracketproductions.com/rustbook/chapter_0.html).

# Running

`cargo run`

# Dependencies

This projects has two main dependencies:
1. The `RLTK` library (now `bracket-lib`), as the name states, is a very handy roguelike toolkit for common features of roguelikes.
   - [https://github.com/amethyst/bracket-lib](https://github.com/amethyst/bracket-lib)
2. The `Specs` library for its powerful Entity Component System.
   - [https://docs.rs/specs/latest/specs/](https://docs.rs/specs/latest/specs/)

See the `Cargo.toml` file for details.

# Misc

Side Note: The Roguelike Tutorial and `bracket-lib` (formerly `RLTK`) have joined the [Amethyst](https://amethyst.rs/) organisation. 
- See [https://amethyst.rs/posts/roguelike-tutorial-and-bracket-lib-joins-amethyst](https://amethyst.rs/posts/roguelike-tutorial-and-bracket-lib-joins-amethyst)