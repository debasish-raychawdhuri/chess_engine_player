# Chess Engine Player

A graphical chess game that allows you to play against any UCI-compatible chess engine.

## Features

- Play against any UCI-compatible chess engine (Stockfish by default)
- Modern GUI interface using Iced framework
- Configurable engine skill level and thinking time
- Option to play as white or black
- Highlighted legal moves
- Game controls: reset, undo moves
- Visual feedback for selected pieces and legal moves

## Usage

```
USAGE:
    chess_engine_player [OPTIONS]

OPTIONS:
    -e, --engine-path <ENGINE_PATH>    Path to the chess engine executable [default: /usr/games/stockfish]
    -s, --skill-level <SKILL_LEVEL>    Engine skill level (1-20) [default: 10]
    -t, --think-time <THINK_TIME>      Engine thinking time in milliseconds [default: 2000]
    -b, --black                        Play as black (engine plays white)
    -h, --help                         Print help information
    -V, --version                      Print version information
```

## Controls

- Click on a piece to select it
- Click on a highlighted square to move the selected piece
- Use the "Reset Game" button to start a new game
- Use the "Undo Move" button to take back the last move pair

## Requirements

- Rust and Cargo
- A UCI-compatible chess engine (Stockfish by default)

## Building and Running

```bash
# Build the project
cargo build

# Run with default settings (Stockfish at skill level 10)
cargo run

# Run with custom settings
cargo run -- --engine-path /path/to/engine --skill-level 15 --think-time 3000 --black
```

## Project Structure

- `src/main.rs`: Application entry point and Iced application
- `src/engine.rs`: Chess engine communication via UCI protocol
- `src/game.rs`: Chess game state and rules
- `src/ui.rs`: Graphical user interface components
- `src/error.rs`: Custom error handling

## License

This project is licensed under the MIT License - see the LICENSE file for details.
