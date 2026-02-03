use chess::{Board, ChessMove, Color, File, Piece, Rank, Square};
use iced::widget::button::Appearance;
use iced::{
    widget::{button, column, container, row, svg, text, Space},
    Alignment, Color as IcedColor, Element, Length,
};

use crate::Message;

// Colors for the chess board
const LIGHT_SQUARE: IcedColor = IcedColor::from_rgb(0.93, 0.93, 0.8);
const DARK_SQUARE: IcedColor = IcedColor::from_rgb(0.46, 0.59, 0.34);
const SELECTED_SQUARE: IcedColor = IcedColor::from_rgb(0.9, 0.8, 0.3);
const LEGAL_MOVE_SQUARE: IcedColor = IcedColor::from_rgb(0.7, 0.9, 0.7);

// Chess UI component
pub struct ChessUI {
    min_board_size: f32,
    max_board_size: f32,
    piece_handles: PieceHandles,
    reset_icon: svg::Handle,
    undo_icon: svg::Handle,
}

// Structure to hold SVG handles for chess pieces
struct PieceHandles {
    white_pawn: svg::Handle,
    white_knight: svg::Handle,
    white_bishop: svg::Handle,
    white_rook: svg::Handle,
    white_queen: svg::Handle,
    white_king: svg::Handle,
    black_pawn: svg::Handle,
    black_knight: svg::Handle,
    black_bishop: svg::Handle,
    black_rook: svg::Handle,
    black_queen: svg::Handle,
    black_king: svg::Handle,
}

impl PieceHandles {
    fn new() -> Self {
        // Load all piece SVGs
        let wp = Self::load_svg("assets/wp.svg");
        let wn = Self::load_svg("assets/wn.svg");
        let wb = Self::load_svg("assets/wb.svg");
        let wr = Self::load_svg("assets/wr.svg");
        let wq = Self::load_svg("assets/wq.svg");
        let wk = Self::load_svg("assets/wk.svg");
        let bp = Self::load_svg("assets/bp.svg");
        let bn = Self::load_svg("assets/bn.svg");
        let bb = Self::load_svg("assets/bb.svg");
        let br = Self::load_svg("assets/br.svg");
        let bq = Self::load_svg("assets/bq.svg");
        let bk = Self::load_svg("assets/bk.svg");

        PieceHandles {
            white_pawn: wp,
            white_knight: wn,
            white_bishop: wb,
            white_rook: wr,
            white_queen: wq,
            white_king: wk,
            black_pawn: bp,
            black_knight: bn,
            black_bishop: bb,
            black_rook: br,
            black_queen: bq,
            black_king: bk,
        }
    }

    fn load_svg(path: &str) -> svg::Handle {
        match std::fs::read(path) {
            Ok(bytes) => {
                println!("Loaded SVG: {}", path);
                svg::Handle::from_memory(bytes)
            }
            Err(e) => {
                eprintln!("Failed to load piece image {}: {}", path, e);
                // Return an empty SVG as fallback
                svg::Handle::from_memory(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" width="45" height="45"></svg>"#
                        .as_bytes()
                        .to_vec(),
                )
            }
        }
    }

    fn get(&self, piece: Piece, color: Color) -> svg::Handle {
        match (piece, color) {
            (Piece::Pawn, Color::White) => self.white_pawn.clone(),
            (Piece::Knight, Color::White) => self.white_knight.clone(),
            (Piece::Bishop, Color::White) => self.white_bishop.clone(),
            (Piece::Rook, Color::White) => self.white_rook.clone(),
            (Piece::Queen, Color::White) => self.white_queen.clone(),
            (Piece::King, Color::White) => self.white_king.clone(),
            (Piece::Pawn, Color::Black) => self.black_pawn.clone(),
            (Piece::Knight, Color::Black) => self.black_knight.clone(),
            (Piece::Bishop, Color::Black) => self.black_bishop.clone(),
            (Piece::Rook, Color::Black) => self.black_rook.clone(),
            (Piece::Queen, Color::Black) => self.black_queen.clone(),
            (Piece::King, Color::Black) => self.black_king.clone(),
        }
    }
}

// Custom style for chess squares
struct ChessSquareStyle {
    color: IcedColor,
}

impl iced::widget::container::StyleSheet for ChessSquareStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.color.into()),
            ..Default::default()
        }
    }
}

// Custom style for side panel with border
struct SidePanelStyle;

impl iced::widget::container::StyleSheet for SidePanelStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(IcedColor::from_rgb(0.15, 0.15, 0.15).into()),
            border_radius: 8.0.into(),
            border_width: 2.0,
            border_color: IcedColor::from_rgb(0.4, 0.4, 0.4),
            ..Default::default()
        }
    }
}

// Custom rounded button style
struct RoundedButtonStyle;

impl iced::widget::button::StyleSheet for RoundedButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(IcedColor::from_rgb(0.3, 0.5, 0.8).into()),
            border_radius: 12.0.into(),
            border_width: 0.0,
            border_color: IcedColor::TRANSPARENT,
            text_color: IcedColor::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(IcedColor::from_rgb(0.4, 0.6, 0.9).into()),
            border_radius: 12.0.into(),
            border_width: 0.0,
            border_color: IcedColor::TRANSPARENT,
            text_color: IcedColor::WHITE,
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(IcedColor::from_rgb(0.25, 0.45, 0.75).into()),
            border_radius: 12.0.into(),
            border_width: 0.0,
            border_color: IcedColor::TRANSPARENT,
            text_color: IcedColor::WHITE,
            ..Default::default()
        }
    }
}

impl ChessUI {
    pub fn new() -> Self {
        ChessUI {
            min_board_size: 320.0,
            max_board_size: 800.0,
            piece_handles: PieceHandles::new(),
            reset_icon: Self::load_icon("assets/reset.svg"),
            undo_icon: Self::load_icon("assets/undo.svg"),
        }
    }

    fn load_icon(path: &str) -> svg::Handle {
        match std::fs::read(path) {
            Ok(bytes) => svg::Handle::from_memory(bytes),
            Err(_) => {
                // Return an empty SVG as fallback
                svg::Handle::from_memory(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"></svg>"#
                        .as_bytes()
                        .to_vec(),
                )
            }
        }
    }

    pub fn view(
        &self,
        board: Board,
        selected_square: Option<Square>,
        possible_moves: &[ChessMove],
        message: &str,
        thinking: bool,
        player_color: Color,
        game_result: Option<chess::GameResult>,
        window_width: u32,
        window_height: u32,
    ) -> Element<'_, Message> {
        // Calculate responsive board size based on window dimensions
        let available_height = window_height as f32 * 0.8; // Use 80% of window height
        let available_width = window_width as f32 * 0.6; // Use 60% of window width
        let board_size = available_height
            .min(available_width)
            .max(self.min_board_size)
            .min(self.max_board_size);

        let square_size = board_size / 8.0;

        // Create a container for the chess board
        let mut board_container = column![];

        // Create rows for the chess board
        for rank in 0..8 {
            let mut row_container = row![];

            for file in 0..8 {
                // Calculate board coordinates based on player color
                let (board_file, board_rank) = if player_color == Color::White {
                    (file, 7 - rank)
                } else {
                    (7 - file, rank)
                };

                let square =
                    Square::make_square(Rank::from_index(board_rank), File::from_index(board_file));

                // Determine square color and state
                let is_dark = (board_rank + board_file) % 2 == 1;
                let is_selected = selected_square == Some(square);
                let is_legal_move = possible_moves.iter().any(|m| m.get_dest() == square);

                let square_color = if is_selected {
                    SELECTED_SQUARE
                } else if is_legal_move {
                    LEGAL_MOVE_SQUARE
                } else if is_dark {
                    DARK_SQUARE
                } else {
                    LIGHT_SQUARE
                };

                // Create square content
                let mut square_content = column![];

                // Add piece if present
                if let Some(piece) = board.piece_on(square) {
                    let piece_color = board.color_on(square).unwrap();

                    // Get SVG handle for the piece
                    let handle = self.piece_handles.get(piece, piece_color);

                    // Add SVG to the square
                    square_content = column![svg(handle)
                        .width(Length::Fixed(square_size * 0.8))
                        .height(Length::Fixed(square_size * 0.8))]
                    .width(Length::Fixed(square_size))
                    .height(Length::Fixed(square_size))
                    .align_items(Alignment::Center);
                }

                // Create clickable square
                let square_element = button(
                    container(square_content)
                        .width(Length::Fixed(square_size))
                        .height(Length::Fixed(square_size))
                        .style(iced::theme::Container::Custom(Box::new(ChessSquareStyle {
                            color: square_color,
                        }))),
                )
                .on_press(Message::SquareClicked(square))
                .padding(0);

                row_container = row_container.push(square_element);
            }

            board_container = board_container.push(row_container);
        }

        // Create the chess board
        let board_view = container(board_container)
            .width(Length::Fixed(board_size))
            .height(Length::Fixed(board_size));

        // Create status message
        let status = if let Some(result) = game_result {
            format!("Game over: {:?}", result)
        } else if thinking {
            "Engine is thinking...".to_string()
        } else {
            format!(
                "{} to move",
                if board.side_to_move() == Color::White {
                    "White"
                } else {
                    "Black"
                }
            )
        };

        // Create player info
        let player_info = format!(
            "You are playing as {}",
            if player_color == Color::White {
                "White"
            } else {
                "Black"
            }
        );

        // Create control buttons with icons and rounded style
        let reset_icon = svg(self.reset_icon.clone())
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0));
        let reset_button = button(
            row![reset_icon, text("Reset")]
                .spacing(5)
                .align_items(Alignment::Center),
        )
        .on_press(Message::ResetGame)
        .padding(10)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle)));

        let undo_icon = svg(self.undo_icon.clone())
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0));
        let undo_button = button(
            row![undo_icon, text("Undo")]
                .spacing(5)
                .align_items(Alignment::Center),
        )
        .on_press(Message::UndoMove)
        .padding(10)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle)));

        // Create the layout
        let controls = row![reset_button, undo_button]
            .spacing(10)
            .padding(10)
            .align_items(Alignment::Center);

        let info_panel = container(
            column![
                text(player_info).size(20),
                text(status).size(16),
                text(message).size(14),
                Space::with_height(Length::Fixed(20.0)),
                controls,
            ]
            .spacing(10)
            .padding(20)
            .align_items(Alignment::Center),
        )
        .width(Length::Fixed(250.0))
        .style(iced::theme::Container::Custom(Box::new(SidePanelStyle)));

        // Combine board and info panel
        let content = row![board_view, info_panel,]
            .spacing(20)
            .padding([10, 20])
            .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
