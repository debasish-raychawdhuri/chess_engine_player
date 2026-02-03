# Chess Engine Player

A beautiful, modern chess application built with Rust that lets you play against any UCI-compatible chess engine (Stockfish by default).

![Chess Engine Player Screenshot](screenshot.png)

## Features

### Gameplay
- **Play against chess engines**: Challenge Stockfish or any UCI-compatible engine
- **Choose your side**: Play as White or Black
- **Adjustable difficulty**: Set engine skill level from 1-20
- **Custom thinking time**: Control how long the engine thinks (100ms to unlimited)

### Visual Interface
- **Modern GUI**: Built with the Iced framework for a clean, responsive interface
- **Piece highlighting**: Selected pieces and legal moves are clearly highlighted
- **Adaptive board**: Board size adjusts to your window
- **Color themes**: Beautiful chess board with distinct light and dark squares
- **Legal move indicators**: Different highlight colors for light and dark squares

### Move History & Analysis
- **Complete move history**: View all moves in Standard Algebraic Notation (SAN)
- **Interactive move list**: Click any move to view that position
- **Position browser**: Navigate through the entire game history
- **Visual feedback**: Active move is highlighted in the move list
- **Table layout**: Clean, aligned display of moves with move numbers

### Game Controls
- **Reset game**: Start a new game anytime
- **Undo moves**: Take back your last move (and the engine's response)
- **Exit view mode**: Return to current position after browsing history

## Installation

### Prerequisites
- Rust and Cargo (latest stable version)
- A UCI-compatible chess engine (Stockfish recommended)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/debasish-raychawdhuri/chess_engine_player.git
cd chess_engine_player

# Build the project
cargo build --release

# Run the application
./target/release/chess_engine_player
```

## Usage

### Basic Usage

```bash
# Play with default settings (Stockfish at skill level 10)
chess_engine_player

# Play as Black
chess_engine_player --black

# Use a different engine
chess_engine_player --engine-path /path/to/your/engine

# Adjust difficulty (1-20)
chess_engine_player --skill-level 15

# Set thinking time in milliseconds
chess_engine_player --think-time 3000
```

### Command Line Options

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

## How to Play

1. **Start the game**: Run the application with your preferred settings
2. **Make a move**: Click on a piece to select it, then click on a highlighted square to move
3. **Wait for the engine**: The engine will automatically calculate and play its move
4. **Browse history**: Click on any move in the side panel to view that position
5. **Continue playing**: Click "Exit View Mode" to return to the current position and continue
6. **Undo mistakes**: Use the "Undo" button to take back moves
7. **Start over**: Use "Reset" to begin a new game

## Move History Features

The move history panel on the right side shows all moves in a clean table format:

- **Move numbers**: Each row shows the move number (1., 2., 3., etc.)
- **White and Black moves**: Displayed side by side for easy reading
- **Click to view**: Click any move to see the board position at that point in the game
- **Active indicator**: The currently viewed move is highlighted in green
- **Scrollable**: The move list scrolls automatically as the game progresses

## Technical Details

### Architecture
- **Language**: Rust
- **GUI Framework**: Iced (immediate mode GUI)
- **Chess Logic**: `chess` crate for move generation and validation
- **Engine Protocol**: UCI (Universal Chess Interface)
- **Async Runtime**: Tokio for asynchronous engine communication

### Project Structure
```
src/
├── main.rs      # Application entry point and Iced app setup
├── engine.rs    # UCI chess engine communication
├── game.rs      # Chess game state, rules, and move history
├── ui.rs        # Graphical user interface components
└── error.rs     # Custom error handling
```

### Supported Platforms
- Linux
- macOS
- Windows

## Configuration

### Using a Different Chess Engine

The application works with any UCI-compatible chess engine. Popular options include:

- **Stockfish** (default): `sudo apt install stockfish`
- **Lc0**: Neural network-based engine
- **Komodo**: Commercial engine with free version available

To use a different engine:
```bash
chess_engine_player --engine-path /usr/local/bin/lc0
```

### Adjusting Engine Strength

The skill level (1-20) controls the engine's playing strength:
- **1-5**: Beginner (good for learning)
- **6-10**: Intermediate
- **11-15**: Advanced
- **16-20**: Expert/Grandmaster level

## Development

### Building
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Chess piece SVGs from [Chess.com](https://www.chess.com/) design resources
- Built with the excellent [Iced](https://iced.rs/) GUI framework
- Chess logic powered by the [chess](https://crates.io/crates/chess) crate
- Stockfish chess engine for providing a world-class opponent

## Screenshots

![Game in Progress](screenshot.png)

*A game in progress showing the move history and legal move highlighting*

---

**Enjoy playing chess against your favorite engine!** ♟️
