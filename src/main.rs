mod engine;
mod error;
mod game;
mod ui;

use std::{
    collections::HashMap,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use clap::Parser;
use iced::{
    executor, window, Application, Command, Element, Event, Settings, Size, Subscription, Theme,
};

use crate::engine::ChessEngine;
use crate::game::{ChessGame, PromotionPiece};
use crate::ui::ChessUI;

// ─── Position Setup State ─────────────────────────────────────────────────────

pub struct SetupState {
    pub pieces: HashMap<chess::Square, (chess::Piece, chess::Color)>,
    pub selected_palette: Option<(chess::Piece, chess::Color)>,
    pub side_to_move: chess::Color,
    pub castle_wk: bool,
    pub castle_wq: bool,
    pub castle_bk: bool,
    pub castle_bq: bool,
    pub en_passant_file: Option<chess::File>,
    pub fen_string: String,
    pub fen_error: Option<String>,
    pub player_color: chess::Color,
}

pub enum AppScreen {
    Game,
    Setup(SetupState),
}

impl SetupState {
    pub fn from_board(board: &chess::Board, player_color: chess::Color) -> Self {
        let mut pieces = HashMap::new();
        for rank_idx in 0..8usize {
            for file_idx in 0..8usize {
                let sq = chess::Square::make_square(
                    chess::Rank::from_index(rank_idx),
                    chess::File::from_index(file_idx),
                );
                if let Some(piece) = board.piece_on(sq) {
                    let color = board.color_on(sq).unwrap();
                    pieces.insert(sq, (piece, color));
                }
            }
        }

        let fen = board.to_string();
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let castling_str = parts.get(2).copied().unwrap_or("-");
        let castle_wk = castling_str.contains('K');
        let castle_wq = castling_str.contains('Q');
        let castle_bk = castling_str.contains('k');
        let castle_bq = castling_str.contains('q');

        let en_passant_file = parts.get(3).and_then(|s| {
            if *s == "-" {
                return None;
            }
            s.chars().next().and_then(|c| {
                let idx = (c as u8).wrapping_sub(b'a') as usize;
                if idx < 8 {
                    Some(chess::File::from_index(idx))
                } else {
                    None
                }
            })
        });

        let side_to_move = board.side_to_move();

        SetupState {
            pieces,
            selected_palette: Some((chess::Piece::Pawn, chess::Color::White)),
            side_to_move,
            castle_wk,
            castle_wq,
            castle_bk,
            castle_bq,
            en_passant_file,
            fen_string: fen,
            fen_error: None,
            player_color,
        }
    }

    pub fn rebuild_fen(&mut self) {
        let mut placement = String::new();
        for rank_idx in (0..8usize).rev() {
            let mut empty: u8 = 0;
            for file_idx in 0..8usize {
                let sq = chess::Square::make_square(
                    chess::Rank::from_index(rank_idx),
                    chess::File::from_index(file_idx),
                );
                if let Some((piece, color)) = self.pieces.get(&sq) {
                    if empty > 0 {
                        placement.push((b'0' + empty) as char);
                        empty = 0;
                    }
                    let c = match (*piece, *color) {
                        (chess::Piece::King,   chess::Color::White) => 'K',
                        (chess::Piece::Queen,  chess::Color::White) => 'Q',
                        (chess::Piece::Rook,   chess::Color::White) => 'R',
                        (chess::Piece::Bishop, chess::Color::White) => 'B',
                        (chess::Piece::Knight, chess::Color::White) => 'N',
                        (chess::Piece::Pawn,   chess::Color::White) => 'P',
                        (chess::Piece::King,   chess::Color::Black) => 'k',
                        (chess::Piece::Queen,  chess::Color::Black) => 'q',
                        (chess::Piece::Rook,   chess::Color::Black) => 'r',
                        (chess::Piece::Bishop, chess::Color::Black) => 'b',
                        (chess::Piece::Knight, chess::Color::Black) => 'n',
                        (chess::Piece::Pawn,   chess::Color::Black) => 'p',
                    };
                    placement.push(c);
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                placement.push((b'0' + empty) as char);
            }
            if rank_idx > 0 {
                placement.push('/');
            }
        }

        let stm = if self.side_to_move == chess::Color::White { "w" } else { "b" };

        // Only include a castling right in the FEN when the relevant king and
        // rook are still on their starting squares; otherwise the chess crate
        // will reject the position as structurally invalid.
        let sq = |rank: usize, file: usize| {
            chess::Square::make_square(
                chess::Rank::from_index(rank),
                chess::File::from_index(file),
            )
        };
        let has = |s: chess::Square, p: chess::Piece, c: chess::Color| {
            self.pieces.get(&s) == Some(&(p, c))
        };
        let wk_home = has(sq(0, 4), chess::Piece::King, chess::Color::White);
        let bk_home = has(sq(7, 4), chess::Piece::King, chess::Color::Black);
        let eff_wk = self.castle_wk && wk_home && has(sq(0, 7), chess::Piece::Rook, chess::Color::White);
        let eff_wq = self.castle_wq && wk_home && has(sq(0, 0), chess::Piece::Rook, chess::Color::White);
        let eff_bk = self.castle_bk && bk_home && has(sq(7, 7), chess::Piece::Rook, chess::Color::Black);
        let eff_bq = self.castle_bq && bk_home && has(sq(7, 0), chess::Piece::Rook, chess::Color::Black);

        let mut castling = String::new();
        if eff_wk { castling.push('K'); }
        if eff_wq { castling.push('Q'); }
        if eff_bk { castling.push('k'); }
        if eff_bq { castling.push('q'); }
        if castling.is_empty() { castling.push('-'); }

        let ep = match self.en_passant_file {
            Some(file) => {
                let file_char = (b'a' + file.to_index() as u8) as char;
                let rank_char = if self.side_to_move == chess::Color::White { '6' } else { '3' };
                format!("{}{}", file_char, rank_char)
            }
            None => "-".to_string(),
        };

        self.fen_string = format!("{} {} {} {} 0 1", placement, stm, castling, ep);

        self.fen_error = safe_parse_board(&self.fen_string).err();
    }

    pub fn parse_fen_to_state(&mut self, fen: &str) {
        self.fen_string = fen.to_string();
        match safe_parse_board(fen) {
            Ok(board) => {
                let mut pieces = HashMap::new();
                for rank_idx in 0..8usize {
                    for file_idx in 0..8usize {
                        let sq = chess::Square::make_square(
                            chess::Rank::from_index(rank_idx),
                            chess::File::from_index(file_idx),
                        );
                        if let Some(piece) = board.piece_on(sq) {
                            let color = board.color_on(sq).unwrap();
                            pieces.insert(sq, (piece, color));
                        }
                    }
                }
                self.pieces = pieces;

                let parts: Vec<&str> = fen.split_whitespace().collect();
                if parts.len() >= 2 {
                    self.side_to_move = if parts[1] == "b" {
                        chess::Color::Black
                    } else {
                        chess::Color::White
                    };
                }
                if parts.len() >= 3 {
                    let c = parts[2];
                    self.castle_wk = c.contains('K');
                    self.castle_wq = c.contains('Q');
                    self.castle_bk = c.contains('k');
                    self.castle_bq = c.contains('q');
                }
                if parts.len() >= 4 {
                    if parts[3] == "-" {
                        self.en_passant_file = None;
                    } else {
                        let idx = (parts[3].chars().next().unwrap_or('-') as u8)
                            .wrapping_sub(b'a') as usize;
                        self.en_passant_file =
                            if idx < 8 { Some(chess::File::from_index(idx)) } else { None };
                    }
                }
                self.fen_error = None;
            }
            Err(e) => {
                self.fen_error = Some(e);
            }
        }
    }
}

/// Parse a FEN string without risking a panic from the chess crate.
///
/// The chess crate aborts on positions it considers structurally broken
/// (missing kings, pawns on the back ranks, etc.).  We catch those cases
/// with cheap string checks before ever handing the FEN to the library.
fn safe_parse_board(fen: &str) -> Result<chess::Board, String> {
    let placement = fen.split_whitespace().next().unwrap_or("");

    // Both kings must be present.
    if !placement.contains('K') {
        return Err("Missing white king (K)".to_string());
    }
    if !placement.contains('k') {
        return Err("Missing black king (k)".to_string());
    }

    // FEN rank order: rank 8 first, rank 1 last.
    let ranks: Vec<&str> = placement.split('/').collect();
    if ranks.len() == 8 {
        if ranks[0].contains('P') || ranks[0].contains('p') {
            return Err("Pawns cannot be on rank 8".to_string());
        }
        if ranks[7].contains('P') || ranks[7].contains('p') {
            return Err("Pawns cannot be on rank 1".to_string());
        }
    }

    chess::Board::from_str(fen).map_err(|e| format!("{:?}", e))
}

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
    screen: AppScreen,
}

// Messages that can be sent to update the application state
#[derive(Debug, Clone)]
pub enum Message {
    SquareClicked(chess::Square),
    ResetGame,
    UndoMove,
    FlipSide,
    EngineMoved(String),
    CheckEngineMove,
    Tick,
    WindowResized(u32, u32),
    ViewMove(usize),
    ExitViewMode,
    ScrollToBottom,
    PromotePawn(PromotionPiece),
    // Setup screen messages
    EnterSetupMode,
    ExitSetupMode,
    SetupSquareClicked(chess::Square),
    SetupPaletteSelected(Option<(chess::Piece, chess::Color)>),
    SetupSideToMove(chess::Color),
    SetupCastlingToggle(u8),
    SetupEnPassant(Option<chess::File>),
    SetupFenChanged(String),
    SetupClearBoard,
    SetupLoadStart,
    SetupStartGame,
    SetupPlayerColor(chess::Color),
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
            screen: AppScreen::Game,
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
                        // select_square returns true for both "move completed"
                        // and "promotion dialog opened".  Don't ask the engine
                        // to move until the player has chosen a promotion piece.
                        if game.pending_promotion().is_some() {
                            return Command::none();
                        }

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

            Message::FlipSide => {
                // Flip the player's side
                let needs_engine_move = if let Ok(mut game) = self.game.lock() {
                    game.flip_side();
                    // Check if it's now the engine's turn
                    game.current_position().side_to_move() != game.player_color()
                } else {
                    false
                };

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
                // Apply the engine's move
                if let Ok(mut game) = self.game.lock() {
                    game.make_engine_move(&best_move);
                    self.engine_thinking = false;
                }
                // Scroll move history to bottom to show latest move
                iced::widget::scrollable::snap_to(
                    iced::widget::scrollable::Id::new("move_history"),
                    iced::widget::scrollable::RelativeOffset::END
                )
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

            Message::PromotePawn(promotion_piece) => {
                if let Ok(mut game) = self.game.lock() {
                    if game.promote_pawn(promotion_piece) {
                        // Move was made, get engine response
                        if game.game_result().is_none() {
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
                }
                Command::none()
            }

            Message::ScrollToBottom => {
                // Scroll move history to bottom - handled by the command
                Command::none()
            }

            // ── Setup screen messages ─────────────────────────────────────
            Message::EnterSetupMode => {
                let (board, player_color) = if let Ok(game) = self.game.lock() {
                    (game.current_position(), game.player_color())
                } else {
                    (chess::Board::default(), chess::Color::White)
                };
                self.screen = AppScreen::Setup(SetupState::from_board(&board, player_color));
                Command::none()
            }

            Message::ExitSetupMode => {
                self.screen = AppScreen::Game;
                Command::none()
            }

            Message::SetupSquareClicked(sq) => {
                if let AppScreen::Setup(ref mut state) = self.screen {
                    match state.selected_palette {
                        Some((piece, color)) => {
                            state.pieces.insert(sq, (piece, color));
                        }
                        None => {
                            state.pieces.remove(&sq);
                        }
                    }
                    state.rebuild_fen();
                }
                Command::none()
            }

            Message::SetupPaletteSelected(palette) => {
                if let AppScreen::Setup(ref mut state) = self.screen {
                    state.selected_palette = palette;
                }
                Command::none()
            }

            Message::SetupSideToMove(color) => {
                if let AppScreen::Setup(ref mut state) = self.screen {
                    state.side_to_move = color;
                    state.rebuild_fen();
                }
                Command::none()
            }

            Message::SetupCastlingToggle(mask) => {
                if let AppScreen::Setup(ref mut state) = self.screen {
                    if mask & 1 != 0 { state.castle_wk = !state.castle_wk; }
                    if mask & 2 != 0 { state.castle_wq = !state.castle_wq; }
                    if mask & 4 != 0 { state.castle_bk = !state.castle_bk; }
                    if mask & 8 != 0 { state.castle_bq = !state.castle_bq; }
                    state.rebuild_fen();
                }
                Command::none()
            }

            Message::SetupEnPassant(file) => {
                if let AppScreen::Setup(ref mut state) = self.screen {
                    state.en_passant_file = file;
                    state.rebuild_fen();
                }
                Command::none()
            }

            Message::SetupFenChanged(fen) => {
                if let AppScreen::Setup(ref mut state) = self.screen {
                    state.parse_fen_to_state(&fen);
                }
                Command::none()
            }

            Message::SetupClearBoard => {
                if let AppScreen::Setup(ref mut state) = self.screen {
                    state.pieces.clear();
                    state.castle_wk = false;
                    state.castle_wq = false;
                    state.castle_bk = false;
                    state.castle_bq = false;
                    state.en_passant_file = None;
                    state.rebuild_fen();
                }
                Command::none()
            }

            Message::SetupLoadStart => {
                if let AppScreen::Setup(ref mut state) = self.screen {
                    let player_color = state.player_color;
                    *state = SetupState::from_board(&chess::Board::default(), player_color);
                }
                Command::none()
            }

            Message::SetupPlayerColor(color) => {
                if let AppScreen::Setup(ref mut state) = self.screen {
                    state.player_color = color;
                }
                Command::none()
            }

            Message::SetupStartGame => {
                let (fen, player_color) = if let AppScreen::Setup(ref state) = self.screen {
                    if state.fen_error.is_some() {
                        return Command::none();
                    }
                    (state.fen_string.clone(), state.player_color)
                } else {
                    return Command::none();
                };

                let needs_engine_move = if let Ok(mut game) = self.game.lock() {
                    game.reset_from_fen(&fen, player_color);
                    game.current_position().side_to_move() != player_color
                } else {
                    false
                };

                self.screen = AppScreen::Game;

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
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            AppScreen::Setup(state) => {
                return self.ui.view_setup(state, self.window_size.width, self.window_size.height);
            }
            AppScreen::Game => {}
        }

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
                game.pending_promotion(),
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
                None,
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
            game_state.10,
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
