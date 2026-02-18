use chess::{Board, ChessMove, Color, File, Piece, Rank, Square};
use iced::widget::button::Appearance;
use iced::widget::scrollable::Scrollable;
use iced::{
    widget::{button, column, container, row, svg, text, Space},
    Alignment, Color as IcedColor, Element, Length,
};

use crate::game::{MoveRecord, PromotionPiece};
use crate::Message;

// Colors for the chess board
const LIGHT_SQUARE: IcedColor = IcedColor::from_rgb(0.93, 0.93, 0.8);
const DARK_SQUARE: IcedColor = IcedColor::from_rgb(0.46, 0.59, 0.34);
const SELECTED_SQUARE: IcedColor = IcedColor::from_rgb(0.9, 0.8, 0.3);
const LEGAL_MOVE_LIGHT_SQUARE: IcedColor = IcedColor::from_rgb(0.7, 0.9, 0.7);
const LEGAL_MOVE_DARK_SQUARE: IcedColor = IcedColor::from_rgb(0.5, 0.75, 0.5);

// Chess UI component
pub struct ChessUI {
    min_board_size: f32,
    max_board_size: f32,
    piece_handles: PieceHandles,
    reset_icon: svg::Handle,
    undo_icon: svg::Handle,
    flip_icon: svg::Handle,
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
        let bytes: Vec<u8> = match path {
            "assets/wp.svg" => include_bytes!("../assets/wp.svg").to_vec(),
            "assets/wn.svg" => include_bytes!("../assets/wn.svg").to_vec(),
            "assets/wb.svg" => include_bytes!("../assets/wb.svg").to_vec(),
            "assets/wr.svg" => include_bytes!("../assets/wr.svg").to_vec(),
            "assets/wq.svg" => include_bytes!("../assets/wq.svg").to_vec(),
            "assets/wk.svg" => include_bytes!("../assets/wk.svg").to_vec(),
            "assets/bp.svg" => include_bytes!("../assets/bp.svg").to_vec(),
            "assets/bn.svg" => include_bytes!("../assets/bn.svg").to_vec(),
            "assets/bb.svg" => include_bytes!("../assets/bb.svg").to_vec(),
            "assets/br.svg" => include_bytes!("../assets/br.svg").to_vec(),
            "assets/bq.svg" => include_bytes!("../assets/bq.svg").to_vec(),
            "assets/bk.svg" => include_bytes!("../assets/bk.svg").to_vec(),
            _ => {
                eprintln!("Unknown SVG path: {}", path);
                return svg::Handle::from_memory(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" width="45" height="45"></svg>"#
                        .as_bytes()
                        .to_vec(),
                );
            }
        };
        svg::Handle::from_memory(bytes)
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

// Custom style for move history buttons
struct MoveHistoryButtonStyle {
    is_active: bool,
}

impl iced::widget::button::StyleSheet for MoveHistoryButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
        if self.is_active {
            Appearance {
                background: Some(IcedColor::from_rgb(0.5, 0.7, 0.3).into()),
                border_radius: 4.0.into(),
                border_width: 0.0,
                border_color: IcedColor::TRANSPARENT,
                text_color: IcedColor::WHITE,
                ..Default::default()
            }
        } else {
            Appearance {
                background: Some(IcedColor::from_rgb(0.4, 0.4, 0.4).into()),
                border_radius: 4.0.into(),
                border_width: 0.0,
                border_color: IcedColor::TRANSPARENT,
                text_color: IcedColor::from_rgb(0.9, 0.9, 0.9),
                ..Default::default()
            }
        }
    }

    fn hovered(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(IcedColor::from_rgb(0.5, 0.5, 0.5).into()),
            border_radius: 4.0.into(),
            border_width: 0.0,
            border_color: IcedColor::TRANSPARENT,
            text_color: IcedColor::WHITE,
            ..Default::default()
        }
    }
}

// Custom style for exit view mode button
struct ExitViewButtonStyle;

impl iced::widget::button::StyleSheet for ExitViewButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(IcedColor::from_rgb(0.8, 0.3, 0.3).into()),
            border_radius: 8.0.into(),
            border_width: 0.0,
            border_color: IcedColor::TRANSPARENT,
            text_color: IcedColor::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(IcedColor::from_rgb(0.9, 0.4, 0.4).into()),
            border_radius: 8.0.into(),
            border_width: 0.0,
            border_color: IcedColor::TRANSPARENT,
            text_color: IcedColor::WHITE,
            ..Default::default()
        }
    }
}

// Custom style for promotion buttons
struct PromotionButtonStyle;

impl iced::widget::button::StyleSheet for PromotionButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(IcedColor::from_rgb(0.2, 0.2, 0.2).into()),
            border_radius: 4.0.into(),
            border_width: 2.0,
            border_color: IcedColor::from_rgb(0.6, 0.6, 0.6),
            text_color: IcedColor::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            background: Some(IcedColor::from_rgb(0.3, 0.3, 0.3).into()),
            border_radius: 4.0.into(),
            border_width: 2.0,
            border_color: IcedColor::from_rgb(0.8, 0.8, 0.8),
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
            flip_icon: Self::load_icon("assets/flip.svg"),
        }
    }

    fn load_icon(path: &str) -> svg::Handle {
        let bytes: Vec<u8> = match path {
            "assets/reset.svg" => include_bytes!("../assets/reset.svg").to_vec(),
            "assets/undo.svg" => include_bytes!("../assets/undo.svg").to_vec(),
            "assets/flip.svg" => include_bytes!("../assets/flip.svg").to_vec(),
            _ => {
                eprintln!("Unknown icon path: {}", path);
                return svg::Handle::from_memory(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"></svg>"#
                        .as_bytes()
                        .to_vec(),
                );
            }
        };
        svg::Handle::from_memory(bytes)
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
        move_records: &[MoveRecord],
        is_view_mode: bool,
        view_move_index: usize,
        pending_promotion: Option<(Square, Square)>,
    ) -> Element<'_, Message> {
        // Calculate responsive board size based on window dimensions
        let available_height = window_height as f32 * 0.9; // Use 90% of window height
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
                    if is_dark {
                        LEGAL_MOVE_DARK_SQUARE
                    } else {
                        LEGAL_MOVE_LIGHT_SQUARE
                    }
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

                    // Add SVG to the square with proper centering
                    square_content = column![
                        Space::with_height(Length::Fixed(square_size * 0.1)),
                        svg(handle)
                            .width(Length::Fixed(square_size * 0.8))
                            .height(Length::Fixed(square_size * 0.8))
                    ]
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
        let status = if is_view_mode {
            format!("Viewing position after move {}", view_move_index)
        } else if let Some(result) = game_result {
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

        let flip_button = button(
            row![
                svg(self.flip_icon.clone())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0)),
                text("Flip")
            ]
            .spacing(5)
            .align_items(Alignment::Center),
        )
        .on_press(Message::FlipSide)
        .padding(10)
        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle)));

        // Create the layout
        let controls = row![reset_button, undo_button, flip_button]
            .spacing(10)
            .padding(10)
            .align_items(Alignment::Center);

        // Build move history display with table-like layout
        let mut move_history_column = column![];
        move_history_column = move_history_column.width(Length::Fill);

        // Fixed widths for columns: move number, white move, black move
        let move_num_width = 35.0;
        let move_btn_width = 100.0;

        for record in move_records {
            let mut move_row = row![];
            move_row = move_row.width(Length::Fill);

            // Move number column (fixed width)
            move_row = move_row.push(
                container(
                    text(format!("{}.", record.move_num))
                        .size(14)
                        .style(IcedColor::from_rgb(0.9, 0.9, 0.9)),
                )
                .width(Length::Fixed(move_num_width))
                .center_y(),
            );

            // White move button column (fixed width, with piece SVG)
            let white_btn: Element<'_, Message> = if let Some(ref white_move) = record.white_move {
                let white_index = record.move_num * 2 - 1;
                let is_white_active = is_view_mode && view_move_index == white_index;

                // Get piece SVG for this move
                let handle = self.piece_handles.get(white_move.piece, Color::White);
                let piece_content: Element<'_, Message> = row![
                    svg(handle)
                        .width(Length::Fixed(22.0))
                        .height(Length::Fixed(22.0)),
                    text(white_move.display_text.clone()).size(14)
                ]
                .spacing(6)
                .align_items(Alignment::Center)
                .into();

                button(container(piece_content).width(Length::Fill).center_y())
                    .on_press(Message::ViewMove(white_index))
                    .padding([4, 8])
                    .width(Length::Fixed(move_btn_width))
                    .style(iced::theme::Button::Custom(Box::new(
                        MoveHistoryButtonStyle {
                            is_active: is_white_active,
                        },
                    )))
                    .into()
            } else {
                // Empty placeholder to maintain alignment
                Space::with_width(Length::Fixed(move_btn_width)).into()
            };
            move_row = move_row.push(white_btn);

            // Black move button column (fixed width, with piece SVG)
            let black_btn: Element<'_, Message> = if let Some(ref black_move) = record.black_move {
                let black_index = record.move_num * 2;
                let is_black_active = is_view_mode && view_move_index == black_index;

                // Get piece SVG for this move
                let handle = self.piece_handles.get(black_move.piece, Color::Black);
                let piece_content: Element<'_, Message> = row![
                    svg(handle)
                        .width(Length::Fixed(22.0))
                        .height(Length::Fixed(22.0)),
                    text(black_move.display_text.clone()).size(14)
                ]
                .spacing(6)
                .align_items(Alignment::Center)
                .into();

                button(container(piece_content).width(Length::Fill).center_y())
                    .on_press(Message::ViewMove(black_index))
                    .padding([4, 8])
                    .width(Length::Fixed(move_btn_width))
                    .style(iced::theme::Button::Custom(Box::new(
                        MoveHistoryButtonStyle {
                            is_active: is_black_active,
                        },
                    )))
                    .into()
            } else {
                // Empty placeholder to maintain alignment
                Space::with_width(Length::Fixed(move_btn_width)).into()
            };
            move_row = move_row.push(black_btn);

            move_history_column = move_history_column
                .push(move_row.spacing(8).align_items(Alignment::Center))
                .push(Space::with_height(Length::Fixed(4.0)));
        }

        // Create scrollable move history with ID for programmatic scrolling
        let move_history_scrollable = Scrollable::new(move_history_column)
            .id(iced::widget::scrollable::Id::new("move_history"))
            .height(Length::Fixed(300.0))
            .width(Length::Fill)
            .style(iced::theme::Scrollable::Default);

        // Create move history section
        let move_history_section = container(
            column![
                text("Move History").size(18),
                Space::with_height(Length::Fixed(10.0)),
                move_history_scrollable,
            ]
            .spacing(5)
            .padding(10),
        )
        .width(Length::Fill)
        .style(iced::theme::Container::Box);

        // Exit view mode button if in view mode
        let exit_view_button = if is_view_mode {
            Some(
                button(text("Exit View Mode").size(14))
                    .on_press(Message::ExitViewMode)
                    .padding([8, 16])
                    .style(iced::theme::Button::Custom(Box::new(ExitViewButtonStyle))),
            )
        } else {
            None
        };

        // Build info panel
        let mut info_panel_content = column![
            text(player_info).size(20),
            text(status).size(16),
            text(message).size(14),
            Space::with_height(Length::Fixed(20.0)),
            controls,
            Space::with_height(Length::Fixed(20.0)),
            move_history_section,
        ]
        .spacing(10)
        .padding(20)
        .align_items(Alignment::Center);

        // Add exit view button if in view mode
        if let Some(btn) = exit_view_button {
            info_panel_content = info_panel_content.push(Space::with_height(Length::Fixed(10.0)));
            info_panel_content = info_panel_content.push(btn);
        }

        let info_panel = container(info_panel_content)
            .width(Length::Fixed(320.0))
            .style(iced::theme::Container::Custom(Box::new(SidePanelStyle)));

        // Promotion dialog
        let promotion_overlay: Option<Element<'_, Message>> = if pending_promotion.is_some() {
            let promotion_buttons: Element<'_, Message> = row![
                button(
                    svg(self.piece_handles.get(Piece::Queen, player_color).clone())
                        .width(Length::Fixed(40.0))
                        .height(Length::Fixed(40.0))
                )
                .on_press(Message::PromotePawn(PromotionPiece::Queen))
                .padding(5)
                .style(iced::theme::Button::Custom(Box::new(PromotionButtonStyle))),
                button(
                    svg(self.piece_handles.get(Piece::Rook, player_color).clone())
                        .width(Length::Fixed(40.0))
                        .height(Length::Fixed(40.0))
                )
                .on_press(Message::PromotePawn(PromotionPiece::Rook))
                .padding(5)
                .style(iced::theme::Button::Custom(Box::new(PromotionButtonStyle))),
                button(
                    svg(self.piece_handles.get(Piece::Bishop, player_color).clone())
                        .width(Length::Fixed(40.0))
                        .height(Length::Fixed(40.0))
                )
                .on_press(Message::PromotePawn(PromotionPiece::Bishop))
                .padding(5)
                .style(iced::theme::Button::Custom(Box::new(PromotionButtonStyle))),
                button(
                    svg(self.piece_handles.get(Piece::Knight, player_color).clone())
                        .width(Length::Fixed(40.0))
                        .height(Length::Fixed(40.0))
                )
                .on_press(Message::PromotePawn(PromotionPiece::Knight))
                .padding(5)
                .style(iced::theme::Button::Custom(Box::new(PromotionButtonStyle))),
            ]
            .spacing(10)
            .padding(15)
            .into();

            let promotion_dialog = container(
                column![
                    text("Promote to:").size(18).style(IcedColor::WHITE),
                    Space::with_height(Length::Fixed(10.0)),
                    promotion_buttons,
                ]
                .align_items(Alignment::Center),
            )
            .style(iced::theme::Container::Custom(Box::new(SidePanelStyle)))
            .width(Length::Fixed(220.0))
            .center_x()
            .center_y();

            Some(
                container(promotion_dialog)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(iced::theme::Container::Box)
                    .into(),
            )
        } else {
            None
        };

        // Combine board and info panel
        let content = if let Some(overlay) = promotion_overlay {
            container(
                column![
                    row![board_view, info_panel,]
                        .spacing(20)
                        .padding([10, 20])
                        .align_items(Alignment::Center),
                    overlay,
                ]
                .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else {
            container(
                row![board_view, info_panel,]
                    .spacing(20)
                    .padding([10, 20])
                    .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        };

        content
    }
}
