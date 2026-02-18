use chess::{Board, ChessMove, Color, File, Game, MoveGen, Piece, Rank, Square};

#[derive(Clone, Debug)]
pub struct MoveDetails {
    #[allow(dead_code)]
    pub notation: String,
    pub piece: Piece,
    #[allow(dead_code)]
    pub destination: String,
    #[allow(dead_code)]
    pub is_capture: bool,
    /// Display text without piece letter (since we show the SVG)
    pub display_text: String,
}

#[derive(Clone, Debug)]
pub struct MoveRecord {
    pub move_num: usize,
    pub white_move: Option<MoveDetails>,
    pub black_move: Option<MoveDetails>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PromotionPiece {
    Queen,
    Rook,
    Bishop,
    Knight,
}

pub struct ChessGame {
    game: Game,
    selected_square: Option<Square>,
    possible_moves: Vec<ChessMove>,
    message: String,
    thinking: bool,
    player_color: Color,
    move_history: Vec<ChessMove>,
    position_history: Vec<Board>,
    move_records: Vec<MoveRecord>,
    view_mode: bool,
    view_move_index: usize,
    pending_promotion: Option<(Square, Square)>,
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
            move_records: Vec::new(),
            view_mode: false,
            view_move_index: 0,
            pending_promotion: None,
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
        self.move_records.clear();
        self.view_mode = false;
        self.view_move_index = 0;
        self.pending_promotion = None;
        self.position_history.push(self.game.current_position());
    }

    pub fn reset_from_fen(&mut self, fen: &str, player_color: Color) {
        use std::str::FromStr;
        match Board::from_str(fen) {
            Ok(board) => {
                self.game = Game::new_with_board(board);
                self.player_color = player_color;
                self.selected_square = None;
                self.possible_moves.clear();
                self.message = String::from("Custom position loaded. Make a move to begin.");
                self.thinking = false;
                self.move_history.clear();
                self.position_history.clear();
                self.move_records.clear();
                self.view_mode = false;
                self.view_move_index = 0;
                self.pending_promotion = None;
                self.position_history.push(board);
            }
            Err(e) => {
                eprintln!("Failed to load position from FEN: {:?}", e);
            }
        }
    }

    pub fn current_position(&self) -> chess::Board {
        if self.view_mode {
            self.position_history[self.view_move_index]
        } else {
            self.game.current_position()
        }
    }

    pub fn is_view_mode(&self) -> bool {
        self.view_mode
    }

    pub fn view_move_index(&self) -> usize {
        self.view_move_index
    }

    pub fn set_view_mode(&mut self, enabled: bool) {
        self.view_mode = enabled;
        if !enabled {
            self.view_move_index = self.position_history.len().saturating_sub(1);
        }
    }

    pub fn view_move_at(&mut self, index: usize) {
        if index < self.position_history.len() {
            self.view_move_index = index;
            self.view_mode = true;
            self.selected_square = None;
            self.possible_moves.clear();
        }
    }

    pub fn get_move_records(&self) -> &Vec<MoveRecord> {
        &self.move_records
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

    pub fn pending_promotion(&self) -> Option<(Square, Square)> {
        self.pending_promotion
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

    pub fn flip_side(&mut self) {
        self.player_color = if self.player_color == Color::White {
            Color::Black
        } else {
            Color::White
        };
        self.message = format!(
            "You are playing as {}.",
            if self.player_color == Color::White {
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

        // Don't allow moves when in view mode
        if self.view_mode {
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
                // MoveGen always produces promotion moves *with* the piece
                // already set (four variants: Q/R/B/N), so get_promotion()
                // is never None for a promotion move.  Detect by checking
                // either the promotion field or the pawn-reaching-back-rank
                // condition so the dialog always fires.
                let needs_promotion = chess_move.get_promotion().is_some()
                    || self.is_pawn_promotion_move(&chess_move, &board);

                if needs_promotion {
                    // Set pending promotion - don't make the move yet
                    self.pending_promotion = Some((chess_move.get_source(), chess_move.get_dest()));
                    self.message = "Select promotion piece".to_string();
                    return true;
                }

                // Get move details BEFORE making the move (need the board position)
                // White is moving (player's turn)
                let details = self.move_to_details(chess_move, &board, Color::White);

                if self.game.make_move(chess_move) {
                    // Save position after making the move
                    self.position_history.push(self.game.current_position());

                    // Record the move
                    self.record_move(details);

                    self.message = format!("Move: {}", chess_move);
                    self.selected_square = None;
                    self.possible_moves.clear();
                    self.move_history.push(chess_move);
                    self.view_move_index = self.position_history.len() - 1;
                    return true;
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

    fn is_pawn_promotion_move(&self, chess_move: &ChessMove, board: &Board) -> bool {
        if let Some(piece) = board.piece_on(chess_move.get_source()) {
            if piece == Piece::Pawn {
                let dest_rank = chess_move.get_dest().get_rank();
                // White pawn promotes on rank 8, black pawn on rank 1
                return (board.side_to_move() == Color::White && dest_rank == Rank::Eighth)
                    || (board.side_to_move() == Color::Black && dest_rank == Rank::First);
            }
        }
        false
    }

    pub fn promote_pawn(&mut self, promotion_piece: PromotionPiece) -> bool {
        if let Some((from, to)) = self.pending_promotion.take() {
            let promotion = match promotion_piece {
                PromotionPiece::Queen => Some(Piece::Queen),
                PromotionPiece::Rook => Some(Piece::Rook),
                PromotionPiece::Bishop => Some(Piece::Bishop),
                PromotionPiece::Knight => Some(Piece::Knight),
            };

            let chess_move = ChessMove::new(from, to, promotion);
            let board = self.game.current_position();
            let details = self.move_to_details(chess_move, &board, Color::White);

            if self.game.make_move(chess_move) {
                self.position_history.push(self.game.current_position());
                self.record_move(details);
                self.message = format!("Move: {}", chess_move);
                self.selected_square = None;
                self.possible_moves.clear();
                self.move_history.push(chess_move);
                self.view_move_index = self.position_history.len() - 1;
                return true;
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
                    // Get move details BEFORE making the move
                    let side_to_move = board.side_to_move();
                    let details = self.move_to_details(m, &board, side_to_move);

                    if self.game.make_move(m) {
                        // Save position after making the move
                        self.position_history.push(self.game.current_position());

                        // Record the move
                        self.record_move(details);

                        self.message = format!("Engine moved: {}", uci_move);
                        self.thinking = false;
                        self.move_history.push(m);
                        self.view_move_index = self.position_history.len() - 1;
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn undo_move_pair(&mut self) {
        if self.thinking {
            // Engine is thinking - undo just the player's last move
            if self.move_history.len() >= 1 && self.position_history.len() >= 2 {
                self.move_history.pop();
                self.position_history.pop();
                self.move_records.pop(); // Remove the incomplete move record
                self.thinking = false;
                self.message = "Undid your move.".to_string();
            } else {
                self.message = "No moves to undo.".to_string();
                return;
            }
        } else {
            // Undo both player and engine moves if possible
            if self.move_history.len() >= 2 && self.position_history.len() >= 3 {
                // Remove the last two moves and positions
                self.move_history.pop();
                self.move_history.pop();
                self.position_history.pop();
                self.position_history.pop();

                // Remove the last complete move record
                self.move_records.pop();

                self.message = "Undid last move pair.".to_string();
            } else if self.move_history.len() >= 1 && self.position_history.len() >= 2 {
                // Only one move to undo
                self.move_history.pop();
                self.position_history.pop();

                // Remove the incomplete move record
                self.move_records.pop();

                self.message = "Undid last move.".to_string();
            } else {
                self.message = "No moves to undo.".to_string();
                return;
            }
        }

        // Restore to the new last position
        let previous_position = self.position_history.last().unwrap().clone();
        self.game = Game::new_with_board(previous_position);

        self.selected_square = None;
        self.possible_moves.clear();
        self.view_move_index = self.position_history.len().saturating_sub(1);
    }

    pub fn game_result(&self) -> Option<chess::GameResult> {
        self.game.result()
    }

    fn move_to_details(&self, chess_move: ChessMove, board: &Board, _side: Color) -> MoveDetails {
        // Get the piece that moved
        let piece = match board.piece_on(chess_move.get_source()) {
            Some(p) => p,
            None => {
                // Fallback to UCI notation if we can't determine the piece
                let from_file =
                    (chess_move.get_source().get_file().to_index() as u8 + b'a') as char;
                let from_rank =
                    (chess_move.get_source().get_rank().to_index() as u8 + b'1') as char;
                let to_file = (chess_move.get_dest().get_file().to_index() as u8 + b'a') as char;
                let to_rank = (chess_move.get_dest().get_rank().to_index() as u8 + b'1') as char;
                let promotion = if let Some(promo) = chess_move.get_promotion() {
                    match promo {
                        Piece::Queen => "q",
                        Piece::Rook => "r",
                        Piece::Bishop => "b",
                        Piece::Knight => "n",
                        _ => "",
                    }
                } else {
                    ""
                };
                let notation = format!(
                    "{}{}{}{}{}",
                    from_file, from_rank, to_file, to_rank, promotion
                );
                let dest_str = format!("{}{}", to_file, to_rank);
                return MoveDetails {
                    notation,
                    piece: Piece::Pawn, // Default fallback
                    destination: dest_str.clone(),
                    is_capture: false,
                    display_text: dest_str,
                };
            }
        };

        let from_file = (chess_move.get_source().get_file().to_index() as u8 + b'a') as char;
        let from_rank = (chess_move.get_source().get_rank().to_index() as u8 + b'1') as char;
        let to_file = (chess_move.get_dest().get_file().to_index() as u8 + b'a') as char;
        let to_rank = (chess_move.get_dest().get_rank().to_index() as u8 + b'1') as char;

        // Check for castling
        if piece == Piece::King {
            let from_file_idx = chess_move.get_source().get_file().to_index();
            let to_file_idx = chess_move.get_dest().get_file().to_index();
            if from_file_idx == 4 {
                if to_file_idx == 6 {
                    return MoveDetails {
                        notation: "O-O".to_string(),
                        piece,
                        destination: "O-O".to_string(),
                        is_capture: false,
                        display_text: "O-O".to_string(),
                    };
                } else if to_file_idx == 2 {
                    return MoveDetails {
                        notation: "O-O-O".to_string(),
                        piece,
                        destination: "O-O-O".to_string(),
                        is_capture: false,
                        display_text: "O-O-O".to_string(),
                    };
                }
            }
        }

        let is_capture = board.piece_on(chess_move.get_dest()).is_some();

        // Build destination string (capture symbol + destination square)
        let capture_symbol = if is_capture { "x" } else { "" };
        let destination = if piece == Piece::Pawn && is_capture {
            // Pawn capture: file + "x" + destination
            format!("{}x{}{}", from_file, to_file, to_rank)
        } else {
            // Other moves: capture symbol + destination
            format!("{}{}{}", capture_symbol, to_file, to_rank)
        };

        // Build full notation with piece letter
        let piece_char = match piece {
            Piece::Pawn => "",
            Piece::Knight => "N",
            Piece::Bishop => "B",
            Piece::Rook => "R",
            Piece::Queen => "Q",
            Piece::King => "K",
        };

        // Check for disambiguation needed (multiple pieces of same type can reach destination)
        let mut disambiguation = String::new();
        if piece != Piece::Pawn && piece != Piece::King {
            let side_to_move = board.side_to_move();
            let move_gen = MoveGen::new_legal(board);
            let mut same_piece_moves = Vec::new();

            for m in move_gen {
                if m.get_dest() == chess_move.get_dest()
                    && m.get_source() != chess_move.get_source()
                {
                    if let Some(other_piece) = board.piece_on(m.get_source()) {
                        if other_piece == piece
                            && board.color_on(m.get_source()) == Some(side_to_move)
                        {
                            same_piece_moves.push(m);
                        }
                    }
                }
            }

            if !same_piece_moves.is_empty() {
                // Check if file is enough to disambiguate
                let file_unique = same_piece_moves
                    .iter()
                    .all(|m| m.get_source().get_file() != chess_move.get_source().get_file());

                if file_unique {
                    // File is different for all other pieces, use file
                    disambiguation.push(from_file);
                } else {
                    // Check if rank is enough
                    let rank_unique = same_piece_moves
                        .iter()
                        .all(|m| m.get_source().get_rank() != chess_move.get_source().get_rank());

                    if rank_unique {
                        // Rank is different for all other pieces, use rank
                        disambiguation.push(from_rank);
                    } else {
                        // Both file and rank needed
                        disambiguation.push(from_file);
                        disambiguation.push(from_rank);
                    }
                }
            }
        }

        let promotion = if let Some(promo_piece) = chess_move.get_promotion() {
            format!(
                "={}",
                match promo_piece {
                    Piece::Queen => "Q",
                    Piece::Rook => "R",
                    Piece::Bishop => "B",
                    Piece::Knight => "N",
                    _ => "",
                }
            )
        } else {
            "".to_string()
        };

        let mut notation = format!(
            "{}{}{}{}",
            piece_char, disambiguation, destination, promotion
        );

        // Build display text without piece letter (since we show the SVG)
        // For pawns, don't include the source file in the destination since we show the piece SVG
        let display_destination = if piece == Piece::Pawn && is_capture {
            format!("x{}{}", to_file, to_rank)
        } else {
            destination.clone()
        };
        let mut display_text = format!("{}{}{}", disambiguation, display_destination, promotion);

        // Check for check or checkmate
        let new_board = board.make_move_new(chess_move);
        if new_board.checkers().popcnt() > 0 {
            let move_gen = MoveGen::new_legal(&new_board);
            if move_gen.len() == 0 {
                notation.push('#');
                display_text.push('#');
            } else {
                notation.push('+');
                display_text.push('+');
            }
        }

        MoveDetails {
            notation,
            piece,
            destination,
            is_capture,
            display_text,
        }
    }

    fn record_move(&mut self, details: MoveDetails) {
        let current_side = self.game.side_to_move();

        if current_side == Color::Black {
            // White just moved, create new record
            let move_num = self.move_records.len() + 1;
            self.move_records.push(MoveRecord {
                move_num,
                white_move: Some(details),
                black_move: None,
            });
        } else {
            // Black just moved, update last record
            if let Some(last_record) = self.move_records.last_mut() {
                last_record.black_move = Some(details);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::{Board, ChessMove, Color, File, Rank, Square};
    use std::str::FromStr;

    #[test]
    fn test_rook_disambiguation_file() {
        // Test position: White rooks on a1 and h1, both can move to d1
        // FEN: 8/8/8/8/8/8/8/R6R w KQ - 0 1
        let board = Board::from_str("8/8/8/8/8/8/8/R6R w KQ - 0 1").unwrap();

        // Create a ChessGame to access move_to_details
        let game = ChessGame::new();

        // Move from a1 to d1
        let from = Square::make_square(Rank::First, File::A);
        let to = Square::make_square(Rank::First, File::D);
        let chess_move = ChessMove::new(from, to, None);

        let details = game.move_to_details(chess_move, &board, Color::White);

        // Should be "Rad1" (file disambiguation needed)
        assert!(
            details.notation.contains("Rad1") || details.notation.contains("Rda1"),
            "Expected Rad1 or similar with file disambiguation, got: {}",
            details.notation
        );
    }

    #[test]
    fn test_rook_disambiguation_rank() {
        // Test position: White rooks on a1 and a8, both can move to a4
        // FEN: R7/8/8/8/8/8/8/R7 w KQ - 0 1
        let board = Board::from_str("R7/8/8/8/8/8/8/R7 w KQ - 0 1").unwrap();

        let game = ChessGame::new();

        // Move from a1 to a4
        let from = Square::make_square(Rank::First, File::A);
        let to = Square::make_square(Rank::Fourth, File::A);
        let chess_move = ChessMove::new(from, to, None);

        let details = game.move_to_details(chess_move, &board, Color::White);

        // Should be "R1a4" (rank disambiguation needed since files are same)
        assert!(
            details.notation.contains("R1a4") || details.notation.contains("Ra14"),
            "Expected R1a4 or similar with rank disambiguation, got: {}",
            details.notation
        );
    }

    #[test]
    fn test_knight_disambiguation_both() {
        // Test position: White knights on b1 and d2, both can move to c3
        // FEN: 8/8/8/8/8/8/3N4/1N6 w KQ - 0 1
        let board = Board::from_str("8/8/8/8/8/8/3N4/1N6 w KQ - 0 1").unwrap();

        let game = ChessGame::new();

        // Move from b1 to c3
        let from = Square::make_square(Rank::First, File::B);
        let to = Square::make_square(Rank::Third, File::C);
        let chess_move = ChessMove::new(from, to, None);

        let details = game.move_to_details(chess_move, &board, Color::White);

        // Should be "Nb1c3" (both file and rank needed since b1 and d2 differ in both)
        // Actually, they differ in file (b vs d), so only file should be needed
        assert!(
            details.notation.starts_with("Nb"),
            "Expected Nb1c3 or Nbc3, got: {}",
            details.notation
        );
    }
}
