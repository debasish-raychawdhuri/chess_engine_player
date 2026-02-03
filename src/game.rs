use chess::{Board, ChessMove, Color, File, Game, MoveGen, Piece, Rank, Square};

pub struct ChessGame {
    game: Game,
    selected_square: Option<Square>,
    possible_moves: Vec<ChessMove>,
    message: String,
    thinking: bool,
    player_color: Color,
    move_history: Vec<ChessMove>,
    position_history: Vec<Board>,
}

impl ChessGame {
    pub fn new() -> Self {
        let mut game = ChessGame {
            game: Game::new(),
            selected_square: None,
            possible_moves: Vec::new(),
            message: String::from("Welcome to Chess Engine Player! Make a move to begin."),
            thinking: false,
            player_color: Color::White,
            move_history: Vec::new(),
            position_history: Vec::new(),
        };

        // Save initial position
        game.position_history.push(game.game.current_position());

        game
    }

    pub fn reset(&mut self) {
        self.game = Game::new();
        self.selected_square = None;
        self.possible_moves.clear();
        self.message = String::from("Game reset. Make a move to begin.");
        self.thinking = false;
        self.move_history.clear();
        self.position_history.clear();
        self.position_history.push(self.game.current_position());
    }

    pub fn current_position(&self) -> chess::Board {
        self.game.current_position()
    }

    pub fn selected_square(&self) -> Option<Square> {
        self.selected_square
    }

    pub fn possible_moves(&self) -> &Vec<ChessMove> {
        &self.possible_moves
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn is_thinking(&self) -> bool {
        self.thinking
    }

    pub fn player_color(&self) -> Color {
        self.player_color
    }

    pub fn set_player_color(&mut self, color: Color) {
        self.player_color = color;
        self.message = format!(
            "You are playing as {}.",
            if color == Color::White {
                "White"
            } else {
                "Black"
            }
        );
    }

    pub fn set_thinking(&mut self, thinking: bool) {
        self.thinking = thinking;
        if thinking {
            self.message = "Engine is thinking...".to_string();
        }
    }

    pub fn select_square(&mut self, square: Square) -> bool {
        // Only allow selecting squares when it's the player's turn
        if self.game.side_to_move() != self.player_color {
            return false;
        }

        let board = self.game.current_position();

        if let Some(_selected) = self.selected_square {
            // If a square is already selected, try to make a move
            let possible_move = self
                .possible_moves
                .iter()
                .find(|m| m.get_dest() == square)
                .cloned();

            if let Some(chess_move) = possible_move {
                // Save current position before making the move
                self.position_history.push(self.game.current_position());

                if self.game.make_move(chess_move) {
                    self.message = format!("Move: {}", chess_move);
                    self.selected_square = None;
                    self.possible_moves.clear();
                    self.move_history.push(chess_move);
                    return true;
                } else {
                    // If move failed, remove the saved position
                    self.position_history.pop();
                }
            } else {
                // Select a new square if it has a piece of the current player's color
                if let Some(_piece) = board.piece_on(square) {
                    if board.color_on(square) == Some(self.game.side_to_move()) {
                        self.selected_square = Some(square);
                        self.update_possible_moves();
                    } else {
                        self.selected_square = None;
                        self.possible_moves.clear();
                    }
                } else {
                    self.selected_square = None;
                    self.possible_moves.clear();
                }
            }
        } else {
            // Select the square if it has a piece of the current player's color
            if let Some(_piece) = board.piece_on(square) {
                if board.color_on(square) == Some(self.game.side_to_move()) {
                    self.selected_square = Some(square);
                    self.update_possible_moves();
                }
            }
        }

        false
    }

    pub fn update_possible_moves(&mut self) {
        self.possible_moves.clear();

        if let Some(square) = self.selected_square {
            let board = self.game.current_position();
            let move_gen = MoveGen::new_legal(&board);

            for m in move_gen {
                if m.get_source() == square {
                    self.possible_moves.push(m);
                }
            }
        }
    }

    pub fn make_engine_move(&mut self, uci_move: &str) -> bool {
        if uci_move.len() < 4 {
            return false;
        }

        let from_file = (uci_move.chars().nth(0).unwrap() as u8 - b'a') as usize;
        let from_rank = (uci_move.chars().nth(1).unwrap() as u8 - b'1') as usize;
        let to_file = (uci_move.chars().nth(2).unwrap() as u8 - b'a') as usize;
        let to_rank = (uci_move.chars().nth(3).unwrap() as u8 - b'1') as usize;

        if from_file >= 8 || from_rank >= 8 || to_file >= 8 || to_rank >= 8 {
            return false;
        }

        let from_square =
            Square::make_square(Rank::from_index(from_rank), File::from_index(from_file));
        let to_square = Square::make_square(Rank::from_index(to_rank), File::from_index(to_file));

        // Find the move in legal moves
        let board = self.game.current_position();
        let move_gen = MoveGen::new_legal(&board);

        for m in move_gen {
            if m.get_source() == from_square && m.get_dest() == to_square {
                // Handle promotion if needed
                let promotion = if uci_move.len() >= 5 {
                    match uci_move.chars().nth(4).unwrap() {
                        'q' => Some(Piece::Queen),
                        'r' => Some(Piece::Rook),
                        'b' => Some(Piece::Bishop),
                        'n' => Some(Piece::Knight),
                        _ => None,
                    }
                } else {
                    None
                };

                if promotion.is_none() || m.get_promotion() == promotion {
                    // Save current position before making the move
                    self.position_history.push(self.game.current_position());

                    if self.game.make_move(m) {
                        self.message = format!("Engine moved: {}", uci_move);
                        self.thinking = false;
                        self.move_history.push(m);
                        return true;
                    } else {
                        // If move failed, remove the saved position
                        self.position_history.pop();
                    }
                }
            }
        }

        false
    }

    pub fn undo_move_pair(&mut self) {
        // Undo both player and engine moves if possible
        if self.move_history.len() >= 2 && self.position_history.len() >= 3 {
            // Go back two moves
            let previous_position = self.position_history[self.position_history.len() - 3].clone();
            self.game = Game::new_with_board(previous_position);

            // Remove the last two moves and positions
            self.move_history.pop();
            self.move_history.pop();
            self.position_history.pop();
            self.position_history.pop();

            self.message = "Undid last move pair.".to_string();
        } else if self.move_history.len() == 1 && self.position_history.len() >= 2 {
            // Go back one move
            let previous_position = self.position_history[self.position_history.len() - 2].clone();
            self.game = Game::new_with_board(previous_position);

            // Remove the last move and position
            self.move_history.pop();
            self.position_history.pop();

            self.message = "Undid last move.".to_string();
        } else {
            self.message = "No moves to undo.".to_string();
        }

        self.selected_square = None;
        self.possible_moves.clear();
    }

    pub fn game_result(&self) -> Option<chess::GameResult> {
        self.game.result()
    }
}
