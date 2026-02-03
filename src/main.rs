mod engine;
mod error;
mod game;
mod ui;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use clap::Parser;
use iced::{
    executor, window, Application, Command, Element, Event, Settings, Size, Subscription, Theme,
};

use crate::engine::ChessEngine;
use crate::game::ChessGame;
use crate::ui::ChessUI;

/// A GUI chess game that allows playing against UCI-compatible chess engines
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the chess engine executable
    #[clap(short, long, default_value = "/usr/games/stockfish")]
    engine_path: PathBuf,

    /// Engine skill level (1-20)
    #[clap(short, long, default_value = "10")]
    skill_level: u8,

    /// Engine thinking time in milliseconds
    #[clap(short, long, default_value = "2000")]
    think_time: u64,

    /// Play as black (engine plays white)
    #[clap(short, long)]
    black: bool,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Validate arguments
    let skill_level = args.skill_level.clamp(1, 20);
    let think_time = args.think_time.max(100);

    // Create settings for the Iced application
    let settings = Settings {
        window: window::Settings {
            size: (1000, 700),
            position: window::Position::Centered,
            min_size: Some((640, 480)),
            max_size: None,
            visible: true,
            resizable: true,
            decorations: true,
            transparent: false,
            ..Default::default()
        },
        default_font: iced::Font::with_name("Noto Sans"),
        flags: AppFlags {
            engine_path: args.engine_path,
            skill_level,
            think_time,
            play_as_black: args.black,
        },
        ..Default::default()
    };

    // Start the Iced application
    ChessApp::run(settings)?;

    Ok(())
}

// Application flags passed from command line
#[derive(Debug, Clone, Default)]
pub struct AppFlags {
    engine_path: PathBuf,
    skill_level: u8,
    think_time: u64,
    play_as_black: bool,
}

// Main application state
pub struct ChessApp {
    game: Arc<Mutex<ChessGame>>,
    engine: Arc<Mutex<ChessEngine>>,
    ui: ChessUI,
    engine_thinking: bool,
    window_size: Size<u32>,
}

// Messages that can be sent to update the application state
#[derive(Debug, Clone)]
pub enum Message {
    SquareClicked(chess::Square),
    ResetGame,
    UndoMove,
    EngineMoved(String),
    CheckEngineMove,
    Tick,
    WindowResized(u32, u32),
    ViewMove(usize),
    ExitViewMode,
}

impl Application for ChessApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = AppFlags;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        // Create game and engine
        let mut game = ChessGame::new();
        let engine = ChessEngine::new();

        // Set player color if playing as black
        if flags.play_as_black {
            game.set_player_color(chess::Color::Black);
        }

        // Create shared state
        let game = Arc::new(Mutex::new(game));
        let engine = Arc::new(Mutex::new(engine));

        // Create UI
        let ui = ChessUI::new();

        // Create application with engine_thinking set if playing as black
        let app = ChessApp {
            game,
            engine,
            ui,
            engine_thinking: flags.play_as_black,
            window_size: Size::new(800, 600),
        };

        // Set thinking state in game if playing as black
        if flags.play_as_black {
            if let Ok(mut game) = app.game.lock() {
                game.set_thinking(true);
            }
        }

        // Start the engine and get first move if playing as black
        let engine_clone = Arc::clone(&app.engine);
        let game_clone = Arc::clone(&app.game);
        let engine_path = flags.engine_path.clone();
        let skill_level = flags.skill_level;
        let think_time = flags.think_time;
        let play_as_black = flags.play_as_black;

        let command = Command::perform(
            async move {
                // Start the engine
                if let Ok(mut engine) = engine_clone.lock() {
                    if let Err(e) = engine.start(&engine_path, skill_level, think_time) {
                        eprintln!("Failed to start engine: {}", e);
                        return false;
                    }
                }

                // If playing as black, get first move from engine
                if play_as_black {
                    // Small delay to ensure engine is ready
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    if let Ok(game) = game_clone.lock() {
                        if let Ok(mut engine) = engine_clone.lock() {
                            let fen = game.current_position().to_string();
                            if let Err(e) = engine.get_move(&fen) {
                                eprintln!("Failed to get engine move: {}", e);
                                return false;
                            }
                        }
                    }
                    return true;
                }
                false
            },
            |needs_check| {
                if needs_check {
                    Message::CheckEngineMove
                } else {
                    Message::Tick
                }
            },
        );

        (app, command)
    }

    fn title(&self) -> String {
        String::from("Chess Engine Player")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SquareClicked(square) => {
                // Handle square click
                if let Ok(mut game) = self.game.lock() {
                    if game.select_square(square) {
                        // Move was made, get engine response
                        if game.game_result().is_none() {
                            self.engine_thinking = true;
                            game.set_thinking(true);

                            // Get engine move
                            let engine_clone = Arc::clone(&self.engine);
                            let game_clone = Arc::clone(&self.game);

                            return Command::perform(
                                async move {
                                    if let Ok(game) = game_clone.lock() {
                                        if let Ok(mut engine) = engine_clone.lock() {
                                            let fen = game.current_position().to_string();
                                            let _ = engine.get_move(&fen);
                                        }
                                    }
                                },
                                |_| Message::CheckEngineMove,
                            );
                        }
                    }
                }
                Command::none()
            }

            Message::ResetGame => {
                // Reset the game
                if let Ok(mut game) = self.game.lock() {
                    game.reset();

                    // If playing as black, get first move from engine
                    if game.player_color() == chess::Color::Black {
                        self.engine_thinking = true;
                        game.set_thinking(true);

                        let engine_clone = Arc::clone(&self.engine);
                        let game_clone = Arc::clone(&self.game);

                        return Command::perform(
                            async move {
                                if let Ok(game) = game_clone.lock() {
                                    if let Ok(mut engine) = engine_clone.lock() {
                                        let fen = game.current_position().to_string();
                                        let _ = engine.get_move(&fen);
                                    }
                                }
                            },
                            |_| Message::CheckEngineMove,
                        );
                    }
                }
                Command::none()
            }

            Message::UndoMove => {
                // Undo the last move pair
                let mut needs_engine_move = false;
                if let Ok(mut game) = self.game.lock() {
                    game.undo_move_pair();
                    // Check if it's the engine's turn after undoing
                    if game.current_position().side_to_move() != game.player_color() {
                        needs_engine_move = true;
                    }
                }
                
                // If it's the engine's turn, trigger engine move
                if needs_engine_move {
                    self.engine_thinking = true;
                    if let Ok(mut game) = self.game.lock() {
                        game.set_thinking(true);
                    }
                    let engine_clone = Arc::clone(&self.engine);
                    let game_clone = Arc::clone(&self.game);
                    
                    return Command::perform(
                        async move {
                            if let Ok(game) = game_clone.lock() {
                                if let Ok(mut engine) = engine_clone.lock() {
                                    let fen = game.current_position().to_string();
                                    let _ = engine.get_move(&fen);
                                }
                            }
                        },
                        |_| Message::CheckEngineMove,
                    );
                }
                
                Command::none()
            }

            Message::EngineMoved(best_move) => {
                // Apply engine move
                if let Ok(mut game) = self.game.lock() {
                    game.make_engine_move(&best_move);
                    self.engine_thinking = false;
                }
                Command::none()
            }

            Message::CheckEngineMove => {
                // Check if engine has a move ready
                if let Ok(engine) = self.engine.lock() {
                    if let Some(best_move) = engine.try_receive_move() {
                        return Command::perform(async { best_move }, Message::EngineMoved);
                    }
                }

                // Schedule another check if engine is still thinking
                if self.engine_thinking {
                    return Command::perform(async {}, |_| Message::CheckEngineMove);
                }

                Command::none()
            }

            Message::Tick => {
                // Regular tick for UI updates
                if self.engine_thinking {
                    return Command::perform(async {}, |_| Message::CheckEngineMove);
                }
                Command::none()
            }

            Message::WindowResized(width, height) => {
                // Update window size
                self.window_size = Size::new(width, height);
                Command::none()
            }

            Message::ViewMove(index) => {
                // View a specific move in history
                if let Ok(mut game) = self.game.lock() {
                    game.view_move_at(index);
                }
                Command::none()
            }

            Message::ExitViewMode => {
                // Exit view mode and return to current position
                if let Ok(mut game) = self.game.lock() {
                    game.set_view_mode(false);
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Get a snapshot of the game state
        let game_state = if let Ok(game) = self.game.lock() {
            (
                game.current_position(),
                game.selected_square(),
                game.possible_moves().clone(),
                game.message().to_string(),
                game.is_thinking(),
                game.player_color(),
                game.game_result(),
                game.get_move_records().clone(),
                game.is_view_mode(),
                game.view_move_index(),
            )
        } else {
            // Default state if lock fails
            (
                chess::Board::default(),
                None,
                Vec::new(),
                "Error accessing game state".to_string(),
                false,
                chess::Color::White,
                None,
                Vec::new(),
                false,
                0,
            )
        };

        // Render the UI with current window size
        self.ui.view(
            game_state.0,
            game_state.1,
            &game_state.2,
            &game_state.3,
            game_state.4,
            game_state.5,
            game_state.6,
            self.window_size.width,
            self.window_size.height,
            &game_state.7,
            game_state.8,
            game_state.9,
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        // Subscribe to time ticks for regular updates and window resize events
        Subscription::batch(vec![
            iced::time::every(std::time::Duration::from_millis(100)).map(|_| Message::Tick),
            iced::subscription::events_with(|event, _| {
                if let Event::Window(window::Event::Resized { width, height }) = event {
                    Some(Message::WindowResized(width, height))
                } else {
                    None
                }
            }),
        ])
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
