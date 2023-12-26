#![feature(exclusive_range_pattern)]
#![allow(unused)]
#![feature(const_trait_impl)]
#![feature(let_chains)]
#![feature(ascii_char)]

extern crate termion;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{Write, stdout, stdin, Stdout, Stdin};
use std::{io, thread, time};
use std::ascii::Char;
use extra::rand::Randomizer;
use std::io::Read;
use std::thread::current;
use termion::async_stdin;
use run_script::ScriptOptions;

mod graphics {
    pub const COORDINATES_X: &'static str = "0    1    2    3    4    5    6    7";
    pub const SEPARATOR_VERTICAL: &'static str = "║";
    pub const SEPARATOR_TOP: &'static str = "╔═══╗╔═══╗╔═══╗╔═══╗╔═══╗╔═══╗╔═══╗╔═══╗";
    pub const SEPARATOR_BOTTOM: &'static str = "╚═══╝╚═══╝╚═══╝╚═══╝╚═══╝╚═══╝╚═══╝╚═══╝";
    pub const BOX_TOP: &'static str = "╔══════════════════════╗";
    pub const BOX_BOTTOM: &'static str = "╚══════════════════════╝";
    pub const PIECES: [&str;13] = [" ", "♖", "♘", "♗", "♕", "♔", "♙", "♜", "♞", "♝", "♛", "♚", "♟︎"];
    pub const START_SCREEN: &'static [&'static str] = &[
        "╔══════════════════════════════╗",
        "║───────CHESS - Mamut──────────║",
        "║──────────────────────────────║",
        "║ h ┆ help                     ║",
        "║ o ┆ mode screen              ║",
        "║ r ┆ restart / new game       ║",
        "║ m ┆ move                     ║",
        "║ q ┆ quit                     ║",
        "║ d ┆ debug                    ║",
        "║ n ┆ switch modes             ║",
        "╚═══╧══════════════════════════╝"
    ];
    pub const HELP_SCREEN: &'static [&'static str] = &[
        "╔═══════════════════════════════════════════════════════════════════╗",
        "║─────────HELP SCREEN───────────────────────────────────────────────║",
        "║───────────────────────────────────────────────────────────────────║",
        "║ X:Y>H:Z    ┆ move piece                                           ║",
        "║            ┆ {X:Y = Row:Column of your piece}                     ║",
        "║            ┆ {H:Z = Row:Column of the square you want to move to} ║",
        "║ s          ┆ start screen                                         ║",
        "╚═══════════════════════════════════════════════════════════════════╝"
    ];
}

#[derive(PartialEq)]
enum Modes {
    AI,
    PvP,
}

#[derive(PartialEq)]
enum MoveInfo {
    Valid,
    InValid,
    BlackCheck,
    WhiteCheck,
    Null
}

#[derive(PartialEq)]
enum GameState {
    Playing,
    Draw,
    WhiteWon,
    BlackWon,
    WhiteCheck,
    BlackCheck,
}

#[derive(PartialEq)]
enum Round {
    White,
    Black
}

struct Game {
    stdout: RawTerminal<Stdout>,
    stdin: Stdin,
    board: [[usize;8];8],
    game_state: GameState,
    round: Round,
    debug: bool,
    wk_moved: bool,
    bk_moved: bool,
    lwr_moved: bool,
    rwr_moved: bool,
    lbr_moved: bool,
    rbr_moved: bool,
    white_captures: Vec<i32>,
    black_captures: Vec<i32>,
    move_info: MoveInfo,
    mode: Modes,
    game_started: bool,
    mode_screen: bool,
    last_en_passant: Vec<i32>,
    move_count: f64,
}

fn draw(x: i32, y: i32, s: String) {

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{}{}{}",
           termion::cursor::Goto(x as u16, y as u16),
           s,
           termion::cursor::Hide).unwrap();
    stdout.flush().unwrap();
}
fn start_screen(se: &mut Game) {

    write!(se.stdout,
           "{}{}{}",
           termion::clear::All,
           termion::cursor::Goto(1, 1),
           termion::cursor::Hide)
        .unwrap();

    se.stdout.flush().unwrap();

    write!(se.stdout,
           "{}",
           termion::clear::All)
        .unwrap();
    se.stdout.flush().unwrap();

    let mut array_counter = 20;

    for x in graphics::START_SCREEN {
        write!(se.stdout, "{}{}{}",
               termion::cursor::Goto(20, array_counter),
               x,
               termion::cursor::Hide).unwrap();
        se.stdout.flush().unwrap();
        array_counter += 1

    }
}
fn help_screen(se: &mut Game) {

    write!(se.stdout,
           "{}{}{}",
           termion::clear::All,
           termion::cursor::Goto(1, 1),
           termion::cursor::Hide)
        .unwrap();


    let mut array_counter = 20;

    se.stdout.flush().unwrap();

    for x in graphics::HELP_SCREEN {
        write!(se.stdout, "{}{}{}",
               termion::cursor::Goto(20, array_counter),
               x,
               termion::cursor::Hide).unwrap();
        se.stdout.flush().unwrap();
        array_counter += 1

    }
}
fn default_setup(se: &mut Game) {
    start_screen(se);
}
fn modes(se: &mut Game) {

    se.mode_screen = true;

    let mut mode_screen = [
        "╔════════════════════════════════════════╗",
        "║─────────SWITCH MODES───────────────────║",
        "║────────────────────────────────────────║",
        "║ Player vs. Player          ┆           ║",
        "║ Player vs. AI  (Stockfish) ┆           ║",
        "╚════════════════════════════════════════╝"
    ];

    write!(se.stdout,
           "{}{}{}",
           termion::clear::All,
           termion::cursor::Goto(1, 1),
           termion::cursor::Hide)
        .unwrap();


    let mut array_counter = 20;

    se.stdout.flush().unwrap();


    for x in mode_screen {
        write!(se.stdout, "{}{}{}",
               termion::cursor::Goto(20, array_counter),
               x,
               termion::cursor::Hide).unwrap();
        se.stdout.flush().unwrap();
        array_counter += 1

    }

    match se.mode {
        Modes::PvP => {
            draw(51, 23, "(current)".to_string());
        }
        Modes::AI => {
            draw(51, 24, "(current)".to_string());
        }
        _ => {}
    }
}
fn game_setup(se: &mut Game) {

    write!(se.stdout,
           "{}{}{}",
           termion::clear::All,
           termion::cursor::Goto(1, 1),
           termion::cursor::Hide)
        .unwrap();

    se.stdout.flush().unwrap();

    display_board(se)
}
fn display_board(se: &mut Game) {

    let mut x_factor: i32 = 25;
    let mut y_factor: i32 = 17;
    let mut coord_x_factor: i32 = 21;
    let mut coord_y_factor: i32 = 20;

    write!(se.stdout,
           "{}{}",
           termion::cursor::Goto(1, 1),
           termion::color::Fg(termion::color::White))
           //termion::clear::All)
        .unwrap();

    //coordinates

    draw(x_factor, y_factor + 1, graphics::COORDINATES_X.to_string());

    for h in 0..8 {
        draw(coord_x_factor, coord_y_factor, h.to_string());
        coord_y_factor += 3
    }

    for q in 0..8 {

        x_factor = 25;
        y_factor += 3;
        draw(x_factor - 2, y_factor - 1, graphics::SEPARATOR_TOP.to_string());
        draw(x_factor - 2, y_factor + 1, graphics::SEPARATOR_BOTTOM.to_string());


        for w in 0..8 {

            //vertical walls
            draw(x_factor - 2, y_factor, graphics::SEPARATOR_VERTICAL.to_string());
            draw(x_factor + 2, y_factor, graphics::SEPARATOR_VERTICAL.to_string());

            match graphics::PIECES[se.board[q as usize][w as usize]] {
                "♖" | "♘" | "♗" | "♕" | "♔" | "♙" => {
                    write!(se.stdout,
                            "{}{}{}{}{}",
                            termion::cursor::Goto(x_factor as u16, y_factor as u16),
                            termion::color::Fg(termion::color::Red),
                            graphics::PIECES[se.board[q as usize][w as usize]],
                            termion::color::Fg(termion::color::White),
                            termion::cursor::Hide)
                        .unwrap();

                },

                "♜" |"♞" | "♝" | "♛" | "♚" | "♟︎" => {
                    write!(se.stdout,
                           "{}{}{}{}{}",
                           termion::cursor::Goto(x_factor as u16, y_factor as u16),
                           termion::color::Fg(termion::color::Green),
                           graphics::PIECES[se.board[q as usize][w as usize]],
                           termion::color::Fg(termion::color::White),
                           termion::cursor::Hide)
                        .unwrap();

                }
                _ => {}
            }

           // draw(x_factor, y_factor, graphics::PIECES[se.board[q as usize][w as usize]].to_string());

            x_factor += 5
        }

        write!(se.stdout,
               "{}{}",
               termion::cursor::Goto(1, 1),
               termion::color::Fg(termion::color::White))
            .unwrap()
    }

    for j in 0..6 {
        if se.round == Round::Black {
            draw(64, 19 + j, "|".to_string());
        } else if se.round == Round::White {
            draw(64, 37 + j, "|".to_string());
        }
    }

    info_board(se);
    captures(se);

    match se.game_state {
        GameState::WhiteWon => {
            end_screen(se);
        }
        GameState::BlackWon => {
            end_screen(se);
        }
        _ => {}
    }

}
fn captures(se: &mut Game) {
    let mut capture_board_white = [
        "╔═════════════════════════════════╗",
        "║───────────────WHITE─────────────║",
        "║ Captured Pieces:                ║",
        "╚═════════════════════════════════╝",
    ];

    let mut capture_board_black = [
        "╔═════════════════════════════════╗",
        "║───────────────BLACK─────────────║",
        "║ Captured Pieces:                ║",
        "╚═════════════════════════════════╝",
    ];

    let mut board_x = 66;
    let mut white_board_y = 27;
    let mut black_board_y = 31;

    for line in capture_board_white {
        write!(se.stdout,
               "{}{}{}",
               termion::cursor::Goto(board_x, white_board_y),
               termion::color::Fg(termion::color::White),
               line)
            .unwrap();

        white_board_y += 1;
    }

    white_board_y = 27;

    write!(se.stdout,
           "{}{}{}",
           termion::cursor::Goto(board_x + 1, white_board_y + 1),
           termion::color::Fg(termion::color::Green),
           "───────────────WHITE─────────────")
        .unwrap();


    for line in capture_board_black {
        write!(se.stdout,
               "{}{}{}",
               termion::cursor::Goto(board_x, black_board_y),
               termion::color::Fg(termion::color::White),
               line)
            .unwrap();

        black_board_y += 1;
    }

    black_board_y = 32;

    write!(se.stdout,
           "{}{}{}",
           termion::cursor::Goto(board_x + 1, black_board_y),
           termion::color::Fg(termion::color::Red),
           "───────────────BLACK─────────────")
        .unwrap();



    let mut pieces_x = 19;

    for piece in &se.white_captures {
        write!(se.stdout,
                "{}{}{}",
                termion::cursor::Goto(board_x + pieces_x, white_board_y + 2),
                termion::color::Fg(termion::color::Red),
                graphics::PIECES[*piece as usize])
            .unwrap();

        pieces_x += 2;
    }

    pieces_x = 19;

    for piece in &se.black_captures {
        write!(se.stdout,
               "{}{}{}",
               termion::cursor::Goto(board_x + pieces_x, black_board_y + 1),
               termion::color::Fg(termion::color::Green),
               graphics::PIECES[*piece as usize])
            .unwrap();

        pieces_x += 2;
    }

    write!(se.stdout,
           "{}{}",
           termion::cursor::Goto(1, 1),
           termion::color::Fg(termion::color::White))
        .unwrap();
}
fn end_screen(se: &mut Game) {

    write!(se.stdout,
           "{}{}{}",
           termion::cursor::Goto(1, 1),
           termion::color::Fg(termion::color::White),
           termion::clear::All)
        .unwrap();

    let mut end_screen = [
        "╔════════════════════════════════╗",
        "║───────────XXXXX WON────────────║",
        "║ Congratulations:               ║",
        "║ Better luck next time:         ║",
        "╚════════════════════════════════╝",
    ];

    let mut end_screen_x = 35;
    let mut end_screen_y = 20;


    match se.game_state {
        GameState::WhiteWon => {
            end_screen = [
                "╔════════════════════════════════╗",
                "║───────────WHITE WON────────────║",
                "║ Congratulations: WHITE         ║",
                "║ Better luck next time: BLACK   ║",
                "╚════════════════════════════════╝",
            ];
        }
        GameState::BlackWon => {
            end_screen = [
                "╔════════════════════════════════╗",
                "║───────────BLACK WON────────────║",
                "║ Congratulations: BLACK         ║",
                "║ Better luck next time: WHITE   ║",
                "╚════════════════════════════════╝",
            ];
        }
        _ => {}
    }


    for line in end_screen {
        write!(se.stdout,
        "{}{}{}",
        termion::cursor::Goto(end_screen_x, end_screen_y),
        termion::color::Fg(termion::color::White),
        line)
        .unwrap();
        end_screen_y += 1;
    }

    let mut x_factor: i32 = 35;
    let mut y_factor: i32 = 26;
    let mut coord_x_factor: i32 = 31;
    let mut coord_y_factor: i32 = 29;

    draw(x_factor, y_factor + 1, graphics::COORDINATES_X.to_string());

    for h in 0..8 {
        draw(coord_x_factor, coord_y_factor, h.to_string());
        coord_y_factor += 3
    }

    for q in 0..8 {
        x_factor = 35;
        y_factor += 3;
        draw(x_factor - 2, y_factor - 1, graphics::SEPARATOR_TOP.to_string());
        draw(x_factor - 2, y_factor + 1, graphics::SEPARATOR_BOTTOM.to_string());


        for w in 0..8 {

            //vertical walls
            draw(x_factor - 2, y_factor, graphics::SEPARATOR_VERTICAL.to_string());
            draw(x_factor + 2, y_factor, graphics::SEPARATOR_VERTICAL.to_string());

            match graphics::PIECES[se.board[q as usize][w as usize]] {
                "♖" | "♘" | "♗" | "♕" | "♔" | "♙" => {
                    write!(se.stdout,
                           "{}{}{}{}{}",
                           termion::cursor::Goto(x_factor as u16, y_factor as u16),
                           termion::color::Fg(termion::color::Red),
                           graphics::PIECES[se.board[q as usize][w as usize]],
                           termion::color::Fg(termion::color::White),
                           termion::cursor::Hide)
                        .unwrap();
                },

                "♜" | "♞" | "♝" | "♛" | "♚" | "♟︎" => {
                    write!(se.stdout,
                           "{}{}{}{}{}",
                           termion::cursor::Goto(x_factor as u16, y_factor as u16),
                           termion::color::Fg(termion::color::Green),
                           graphics::PIECES[se.board[q as usize][w as usize]],
                           termion::color::Fg(termion::color::White),
                           termion::cursor::Hide)
                        .unwrap();
                }
                _ => {}
            }

            // draw(x_factor, y_factor, graphics::PIECES[se.board[q as usize][w as usize]].to_string());

            x_factor += 5
        }
    }
}
fn game(se: &mut Game) {

    se.game_started = true;

    game_setup(se);
}
fn info_board(se: &mut Game) {

    let mut board = [
        "╔═════════════════════════════════╗",
        "║───────────────INFO──────────────║",
        "║ Game State:                     ║",
        "║ Round:                          ║",
        "║ Debug State:                    ║",
        "║ Move Info:                      ║",
        "║ Mode:                           ║",
        "╚═════════════════════════════════╝",
    ];

    let mut info_board_x = 66;
    let mut info_board_y = 19;

    for line in board {
        write!(se.stdout,
                "{}{}{}",
                termion::cursor::Goto(info_board_x, info_board_y),
                termion::color::Fg(termion::color::White),
                line)
            .unwrap();
        info_board_y += 1;
    }

    info_board_x = 66;
    info_board_y = 19;

    match se.game_state {
        GameState::Playing => {
            write!(se.stdout,
                    "{}{}{}",
                    termion::cursor::Goto(info_board_x as u16 + 14, info_board_y as u16 + 2),
                    termion::color::Fg(termion::color::Red),
                    "playing")
                .unwrap();
            },
        GameState::Draw => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 14, info_board_y as u16 + 2),
                   termion::color::Fg(termion::color::Red),
                   "draw")
                .unwrap();
        },
        GameState::WhiteWon => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 14, info_board_y as u16 + 2),
                   termion::color::Fg(termion::color::Red),
                   "white won")
                .unwrap();
        },
        GameState::BlackWon => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 14, info_board_y as u16 + 2),
                   termion::color::Fg(termion::color::Red),
                   "black won")
                .unwrap();
        },GameState::WhiteCheck => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 14, info_board_y as u16 + 2),
                   termion::color::Fg(termion::color::Red),
                   "white check")
                .unwrap();
        },
        GameState::BlackCheck => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 14, info_board_y as u16 + 2),
                   termion::color::Fg(termion::color::Red),
                   "black check")
                .unwrap();
        }
        _ => {},
    }
    match se.round {

        Round::White => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 9, info_board_y as u16 + 3),
                   termion::color::Fg(termion::color::Green),
                   "white")
                .unwrap();
        },

        Round::Black => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 9, info_board_y as u16 +3),
                   termion::color::Fg(termion::color::Red),
                   "black")
                .unwrap();
        },
        _ => {}

    }
    match se.debug {
        true => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 15, info_board_y as u16 + 4),
                   termion::color::Fg(termion::color::Green),
                   "true")
                .unwrap();
        },
        false => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 15, info_board_y as u16 + 4),
                   termion::color::Fg(termion::color::Red),
                   "false")
                .unwrap();
        },
        _ => {}
    }
    match se.move_info {
        MoveInfo::Null => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 13, info_board_y as u16 + 5),
                   termion::color::Fg(termion::color::White),
                   "No move")
                .unwrap();

        }
        MoveInfo::Valid => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 13, info_board_y as u16 + 5),
                   termion::color::Fg(termion::color::Green),
                   "Valid move")
                .unwrap();

        }
        MoveInfo::InValid => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 13, info_board_y as u16 + 5),
                   termion::color::Fg(termion::color::Red),
                   "Invalid move")
                .unwrap();

        }
        MoveInfo::BlackCheck => {
            write!(se.stdout,
                   "{}{}{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 13, info_board_y as u16 + 5),
                   termion::color::Fg(termion::color::Red),
                   "Move doesn't",
                   termion::cursor::Goto(info_board_x as u16 + 2, info_board_y as u16 + 6),
                   "protect the black king")
            .unwrap();

        }
        MoveInfo::WhiteCheck => {
            write!(se.stdout,
                   "{}{}{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 13, info_board_y as u16 + 5),
                   termion::color::Fg(termion::color::Red),
                   "Move doesn't",
                   termion::cursor::Goto(info_board_x as u16 + 2, info_board_y as u16 + 6),
                   "protect the white king")
                .unwrap();

        }
        _ => {}
    }
    match se.mode {
        Modes::PvP => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 8, info_board_y as u16 + 6),
                   termion::color::Fg(termion::color::White),
                   "Player vs. Player")
                .unwrap();
        }
        Modes::AI => {
            write!(se.stdout,
                   "{}{}{}",
                   termion::cursor::Goto(info_board_x as u16 + 8, info_board_y as u16 + 6),
                   termion::color::Fg(termion::color::White),
                   "Player vs. AI")
                .unwrap();
        }
        _ => {}
    }
    write!(se.stdout,
            "{}{}",
            termion::cursor::Goto(1, 1),
            termion::color::Fg(termion::color::White))
        .unwrap()


}
fn move_piece(se: &mut Game) {

    draw(23, 14, graphics::BOX_TOP.to_string());
    draw(23, 16, graphics::BOX_BOTTOM.to_string());
    draw(23, 15, graphics::SEPARATOR_VERTICAL.to_string());
    draw(46, 15, graphics::SEPARATOR_VERTICAL.to_string());
    draw(25, 15, "MOVE : ".to_string());

    let mut command = get_command(se);

    if valid(command.clone()) {
        let (og_r, og_c, n_r, n_c) = parse_cmd(command);
        if can_move(se, og_r, og_c, n_r, n_c) {
            if se.mode == Modes::PvP {
                let mut old_board = se.board;
                let og = se.board[og_r as usize][og_c as usize];
                let piece = se.board[n_r as usize][n_c as usize];

                //Black Rochades
                if se.board[og_r as usize][og_c as usize] == 5 && se.board[n_r as usize][n_c as usize] == 1 {
                    match n_c {
                        0 => {
                            se.board[0][4] = 1;
                            se.board[0][0] = 5;
                        }
                        7 => {
                            se.board[0][4] = 1;
                            se.board[0][7] = 5;
//                             se.board[0][5] = 1;
//                             se.board[0][6] = 5;
                        }
                        _ => {}
                    }
                // White Rochades
                } else if se.board[og_r as usize][og_c as usize] == 11 && se.board[n_r as usize][n_c as usize] == 7 {
                    match n_c {
                        0 => {
                            se.board[7][4] = 7;
                            se.board[7][0] = 11;
                        }
                        7 => {
                            se.board[7][4] = 7;
                            se.board[7][7] = 11;
//                             se.board[7][5] = 7;
//                             se.board[7][6] = 11;
                        }
                        _ => {}
                    }
                // Normal Move
                } else {
                    se.board[n_r as usize][n_c as usize] = og;
                    se.board[og_r as usize][og_c as usize] = 0;
                }

                if se.round == Round::White {
                    // Check for white checks
                    if check(se, 11) {
                        se.board = old_board;
                        se.round = Round::Black;
                        se.move_info = MoveInfo::WhiteCheck;
                        se.game_state = GameState::Playing;
                    } else {
                        se.game_state = GameState::Playing;
                        se.move_info = MoveInfo::Valid;

                        // Check for black checks and for black mates
                        if check(se, 5) {
                            if mate(se, 5) {
                                se.game_state = GameState::WhiteWon;
                            } else {
                                se.game_state = GameState::BlackCheck
                            }
                        }


                        // En passant protocol
                        if old_board[og_r as usize][og_c as usize] == 6 && n_r - 2 == og_r {
                            se.last_en_passant = vec![n_r as i32 - 1, n_c as i32];
                        } else if old_board[og_r as usize][og_c as usize] == 12 && n_r + 2 == og_r {
                            se.last_en_passant = vec![n_r as i32 + 1, n_c as i32];
                        }

                        se.move_count += 0.5;

                        // Promoting
                        if old_board[og_r as usize][og_c as usize] == 12 {
                            if n_r == 0 {
                                let mut pie = promoting_screen(se);
                                match pie {
                                    1 => {
                                        pie = 7;
                                    }
                                    2 => {
                                        pie = 8;
                                    }
                                    3 => {
                                        pie = 9;
                                    }
                                    4 => {
                                        pie = 10;
                                    }
                                    _ => {}
                                }
                                se.board[n_r as usize][n_c as usize] = pie as usize;
                            }
                        }
                    }
                } else if se.round == Round::Black {

                    // Check for black checks
                    if check(se, 5) {
                        se.board = old_board;
                        se.round = Round::White;
                        se.move_info = MoveInfo::BlackCheck;
                        se.game_state = GameState::Playing;
                    } else {
                        se.game_state = GameState::Playing;
                        se.move_info = MoveInfo::Valid;

                        // Check for black checks and for black mates
                        if check(se, 11) {
                            if mate(se, 11) {
                                se.game_state = GameState::BlackWon;
                            } else {
                                se.game_state = GameState::WhiteCheck
                            }
                        }

                        // En passant protocol
                        if old_board[og_r as usize][og_c as usize] == 6 && n_r - 2 == og_r {
                            se.last_en_passant = vec![n_r as i32 - 1, n_c as i32];
                        } else if old_board[og_r as usize][og_c as usize] == 12 && n_r + 2 == og_r {
                            se.last_en_passant = vec![n_r as i32 + 1, n_c as i32];
                        }

                        se.move_count += 0.5;

                        // Promoting
                        if old_board[og_r as usize][og_c as usize] == 6 {
                            if n_r == 7 {
                                let mut pie = promoting_screen(se);
                                match pie {
                                    1 => {
                                        pie = 1;
                                    }
                                    2 => {
                                        pie = 2;
                                    }
                                    3 => {
                                        pie = 3;
                                    }
                                    4 => {
                                        pie = 4;
                                    }
                                    _ => {}
                                }
                                se.board[n_r as usize][n_c as usize] = pie as usize;
                            }
                        }
                    }
                }


                if old_board != se.board {
                    match old_board[og_r as usize][og_c as usize] {
                        1 | 2 | 3 | 4 | 5 | 6 => {
                            match old_board[n_r as usize][n_c as usize] {
                                7 | 8 | 9 | 10 | 11 | 12 => {
                                    se.black_captures.push(old_board[n_r as usize][n_c as usize] as i32);
                                }
                                _ => {}
                            }
                        }
                        7 | 8 | 9 | 10 | 11 | 12 => {
                            match old_board[n_r as usize][n_c as usize] {
                                1 | 2 | 3 | 4 | 5 | 6 => {
                                    se.white_captures.push(old_board[n_r as usize][n_c as usize] as i32);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }

                if se.round == Round::White {
                    se.round = Round::Black
                } else if se.round == Round::Black {
                    se.round = Round::White
                }

                if se.board != old_board {
                    if se.board[0][0] != 1 {
                        se.lbr_moved = true;
                    } else if se.board[0][7] != 1 {
                        se.rbr_moved = true;
                    } else if se.board[7][0] != 7 {
                        se.lwr_moved = true;
                    } else if se.board[7][7] != 7 {
                        se.rwr_moved = true;
                    }
                }


                if se.debug == true {
                    write!(se.stdout,
                           "{}{:?}",
                           termion::cursor::Goto(10, 50),
                           se.board)
                        .unwrap();

                }



                // TODO: Rewrite the whole mechanism so that when it's the white turn it check's for black checks, it checks for white checks in wich case it reverts back to normal and it check if a move that has not triggered a white check is a promotion in which case it triggers a promotion screen
            } else if se.mode == Modes::AI {
                let mut old_board = se.board;
                let og = se.board[og_r as usize][og_c as usize];
                let piece = se.board[n_r as usize][n_c as usize];

                //Black Rochades
                if se.board[og_r as usize][og_c as usize] == 5 && se.board[n_r as usize][n_c as usize] == 1 {
                    match n_c {
                        0 => {
                            se.board[0][4] = 1;
                            se.board[0][0] = 5;
                        }
                        7 => {
                            se.board[0][4] = 1;
                            se.board[0][7] = 5;
//                             se.board[0][5] = 1;
//                             se.board[0][6] = 5;
                        }
                        _ => {}
                    }
                } else if se.board[og_r as usize][og_c as usize] == 11 && se.board[n_r as usize][n_c as usize] == 7 {
                    match n_c {
                        0 => {
                            se.board[7][4] = 7;
                            se.board[7][0] = 11;
                        }
                        7 => {
                            se.board[7][4] = 7;
                            se.board[7][7] = 11;
//                             se.board[7][5] = 7;
//                             se.board[7][6] = 11;
                        }
                        _ => {}
                    }

                } else {
                    se.board[n_r as usize][n_c as usize] = og;
                    se.board[og_r as usize][og_c as usize] = 0;
                }

                if check(se, 5) != true && check(se, 11) != true {
                    // so there is no check on the black king as no check on the white king

                    se.game_state = GameState::Playing;
                    se.move_info = MoveInfo::Valid;

                    if old_board[og_r as usize][og_c as usize] == 6 && n_r - 2 == og_r {
                        se.last_en_passant = vec![n_r as i32 - 1, n_c as i32];
                    } else if old_board[og_r as usize][og_c as usize] == 12 && n_r + 2 == og_r {
                        se.last_en_passant = vec![n_r as i32 + 1, n_c as i32];
                    }

                    se.move_count += 0.5;




                }

                if se.round == Round::White {
                    if check(se, 11) {

                        se.move_info = MoveInfo::WhiteCheck;
                        if old_board[og_r as usize][og_c as usize] == 11 && old_board[n_r as usize][n_c as usize] == 7 {
                            match n_c {
                                0 => {
                                    old_board[7][4] = 11;
                                    old_board[7][0] = 7;
                                }
                                7 => {
                                    old_board[7][4] = 11;
                                    old_board[7][7] = 7;
                                    old_board[7][5] = 0;
                                    old_board[7][6] = 0;
                                }
                                _ => {}
                            }
                            se.game_state = GameState::Playing;
                        }
                        se.round = Round::Black;
                        se.board = old_board;
                    }
                } else if se.round == Round::Black {
                    if check(se, 5) {
                        se.move_info = MoveInfo::BlackCheck;
                        if old_board[og_r as usize][og_c as usize] == 5 && old_board[n_r as usize][n_c as usize] == 1 {
                            match n_c {
                                0 => {
                                    old_board[0][0] = 1;
                                    old_board[0][4] = 5;
                                }
                                7 => {
                                    old_board[0][4] = 5;
                                    old_board[0][7] = 1;
                                    old_board[0][5] = 0;
                                    old_board[0][6] = 0;
                                }
                                _ => {}
                            }
                            se.game_state = GameState::Playing;
                        }
                        se.round = Round::White;
                        se.board = old_board;
                    }
                }

                if old_board != se.board {
                    match old_board[og_r as usize][og_c as usize] {
                        1 | 2 | 3 | 4 | 5 | 6 => {
                            match old_board[n_r as usize][n_c as usize] {
                                7 | 8 | 9 | 10 | 11 | 12 => {
                                    se.black_captures.push(old_board[n_r as usize][n_c as usize] as i32);
                                }
                                _ => {}
                            }
                        }
                        7 | 8 | 9 | 10 | 11 | 12 => {
                            match old_board[n_r as usize][n_c as usize] {
                                1 | 2 | 3 | 4 | 5 | 6 => {
                                    se.white_captures.push(old_board[n_r as usize][n_c as usize] as i32);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }

                if check(se, 5) {
                    if mate(se, 5) {
                        se.game_state = GameState::WhiteWon;
                    }
                } else if check(se, 11) {
                    if mate(se, 11) {
                        se.game_state = GameState::BlackWon;
                    }
                }


                if se.board != old_board {
                    if se.board[0][0] != 1 {
                        se.lbr_moved = true;
                    } else if se.board[0][7] != 1 {
                        se.rbr_moved = true;
                    } else if se.board[7][0] != 7 {
                        se.lwr_moved = true;
                    } else if se.board[7][7] != 7 {
                        se.rwr_moved = true;
                    }
                }


                if se.debug == true {
                    write!(se.stdout,
                           "{}{:?}",
                           termion::cursor::Goto(10, 50),
                           se.board)
                        .unwrap();

                }

                display_board(se);

                if check(se, 5) == false && check(se, 11) == false {
                    // AI

                    se.round = Round::Black;

                    let fen_board = board_to_fen(se);
                    let best_move = get_best_move(fen_board.clone());

                    draw(17, 10, fen_board.clone());
                    draw(18, 11, best_move.clone());


                    println!("{}", fen_board);
                    println!("{}", best_move);

                    let m1 = best_move.chars().nth(0).unwrap().to_string() + &best_move.chars().nth(1).unwrap().to_string() as &str;
                    let m2 = best_move.chars().nth(2).unwrap().to_string() + &best_move.chars().nth(3).unwrap().to_string() as &str;


                    let x = fen_cmd_to_bo_cmd(m1.to_string());
                    let y = fen_cmd_to_bo_cmd(m2.to_string());


                    if se.debug {
                        write!(se.stdout,
                                "{}{:?}{}{:?}",
                                termion::cursor::Goto(15, 40),
                                x,
                                termion::cursor::Goto(16, 40),
                                y)
                            .unwrap()
                    }

                    let og_char = se.board[x[0] as usize][x[1] as usize];

                    se.board[y[0] as usize][y[1] as usize] = og_char;
                    se.board[x[0] as usize][x[1] as usize] = 0;

                    se.move_count += 0.5;

                    write!(se.stdout,
                            "{}{}",
                            termion::cursor::Goto(1, 1),
                            termion::clear::All)
                        .unwrap();


                    se.round = Round::White;

                }

            }

        } else {
        se.move_info = MoveInfo::InValid;
    }

    } else {
        se.move_info = MoveInfo::InValid;
    }

    display_board(se);

}
fn valid(cmd: String) -> bool {

    let possible_numbers: [char; 8] = ["0".parse().unwrap(), "1".parse().unwrap(), "2".parse().unwrap(), "3".parse().unwrap(), "4".parse().unwrap(), "5".parse().unwrap(), "6".parse().unwrap(), "7".parse().unwrap()];

    // Check if cmd has x:x>y:y
    if cmd.len() != 7  {
        return false
    }

    // Check if cmd >= 0 & cmd <= 7

    for ch in cmd.chars() {
        match ch {
            '-' => return false,
            '8' | '9' => return false,
            _ => {}
        }
    }

    // Check if the format if appropriate

    for f in 0..cmd.len() {
        let mut char = cmd.chars().nth(f).unwrap();
        match f {
            0 => {
                if !contains_07(char) {
                    return false
                }
            }
            1 => {
                if char != ":".parse().unwrap() {
                    return false
                }
            },
            2 => {
                if !contains_07(char) {
                    return false
                }
            },
            3 => {
                if char != ">".parse().unwrap() {
                    return false
                }
            },
            4 => {
                if !contains_07(char) {
                    return false
                }
            },
            5 => {
                if char != ":".parse().unwrap() {
                    return false
                }
            },
            6 => {
                if !contains_07(char) {
                    return false
                }
            },

            _ => {}
        }
    }

    return true
}
fn contains_07(char: char) -> bool {

    if char != "0".parse().unwrap() && char != "1".parse().unwrap() && char != "2".parse().unwrap() && char != "3".parse().unwrap() && char != "4".parse().unwrap() && char != "5".parse().unwrap() && char != "6".parse().unwrap() && char != "7".parse().unwrap() {
        return false
    }
    return true
}
fn parse_cmd(cmd: String) -> (u32, u32, u32, u32) {

    let og_r = cmd.chars().nth(0).unwrap().to_digit(10).unwrap();
    let og_c = cmd.chars().nth(2).unwrap().to_digit(10).unwrap();

    let n_r = cmd.chars().nth(4).unwrap().to_digit(10).unwrap();
    let n_c = cmd.chars().nth(6).unwrap().to_digit(10).unwrap();



    return(og_r, og_c, n_r, n_c)
}
fn get_command(se: &mut Game) -> String {

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();


    let mut command: String = "".to_string();


    for c in stdin.keys() {
        write!(se.stdout,
               "{}{}{}",
               termion::cursor::Goto(32, 15),
               command,
               termion::cursor::Hide)
            .unwrap();

        match c.unwrap() {
            Key::Char('q') => {
                write!(se.stdout,
                        "{}{}",
                        termion::clear::All,
                        termion::cursor::Hide)
                    .unwrap();
                break
            },
            Key::Char('e') => {
                write!(se.stdout,
                       "{}{}",
                       termion::clear::All,
                       termion::cursor::Hide)
                    .unwrap();
                break
            },
            Key::Char(c) => {
                command += &c.to_string();
                write!(se.stdout,
                       "{}{}{}",
                       termion::cursor::Goto(32, 15),
                       command,
                       termion::cursor::Hide)
                    .unwrap();
            }
            _ => {}
        }
        se.stdout.flush().unwrap();
    }

    write!(se.stdout, "{}", termion::cursor::Hide).unwrap();

    return command;

}
fn can_move(se: &mut Game, or: u32, oc: u32, nr: u32, nc: u32) -> bool {

    if se.board[or as usize][oc as usize] == 0 {
        se.move_info = MoveInfo::InValid;
        return false
    }

    match se.board[or as usize][oc as usize] {
        1 | 2 | 3 | 4 | 5 | 6 => {
            // Black

            if se.round == Round::White {
                se.move_info = MoveInfo::InValid;
                return false
            }
        }

        7 | 8 | 9 | 10 | 11 | 12 => {
            // White

            if se.round == Round::Black {
                se.move_info = MoveInfo::InValid;
                return false
            }
        }
        _ => {}
    }
    //Black Rochades
    if se.game_state != GameState::BlackCheck {
        if se.board[or as usize][oc as usize] == 5 && se.board[nr as usize][nc as usize] == 1 && se.bk_moved == false {
            match nc {
                0 => {
                    if se.lbr_moved == false && se.board[0][1] == 0 && se.board[0][2] == 0 && se.board[0][3] == 0 {
                        return true
                    }
                }
                7 => {
                    if se.rbr_moved == false  && se.board[0][5] == 0 && se.board[0][6] == 0 {
                        return true
                    }
                }
                _ => {}
            }
        }
    }
        //White Rochades
    if se.game_state != GameState::WhiteCheck {
        if se.board[or as usize][oc as usize] == 11 && se.board[nr as usize][nc as usize] == 7 && se.wk_moved == false {
            match nc {
                0 => {
                    if se.lwr_moved == false && se.board[7][1] == 0 && se.board[7][2] == 0 && se.board[7][3] == 0 {
                        return true
                    }
                }
                7 => {
                    if se.rwr_moved == false && se.board[7][5] == 0 && se.board[7][6] == 0 {
                        return true
                    }
                }
                _ => {}
            }
        }

        match se.board[or as usize][oc as usize] {
            1 | 2 | 3 | 4 | 5 | 6 => {
                match se.board[nr as usize][nc as usize] {
                    1 | 2 | 3 | 4 | 5 | 6 => {
                        se.move_info = MoveInfo::InValid;
                        return false
                    }
                    _ => {}
                }
            }
            7 | 8 | 9 | 10 | 11 | 12 => {
                match se.board[nr as usize][nc as usize] {
                    7 | 8 | 9 | 10 | 11 | 12 => {
                        se.move_info = MoveInfo::InValid;
                        return false
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let mut pos_vec: Vec<i32> = vec![nr as i32, nc as i32];

    let pos_moves = get_pos_moves(se, se.board[or as usize][oc as usize] as i32, or as i32, oc as i32);



    return if pos_moves.contains(&pos_vec) {
        if se.debug == true {
            write!(se.stdout,
                    "{}{:?}",
                    termion::cursor::Goto(10, 10),
                    pos_vec)
                .unwrap();

            write!(se.stdout,
                    "{}{:?}",
                    termion::cursor::Goto(10, 11),
                    pos_moves)
                .unwrap();

        }

        true
    } else {
        se.move_info = MoveInfo::InValid;
        false
    }



}
fn check(se: &mut Game, k: i32) -> bool {

    // Getting king coords

    let mut wk: Vec<i32> = vec![];
    let mut bk: Vec<i32> = vec![];

    for row in 0..8 {
        for col in 0..8 {
            let mut coord = vec![row as i32, col as i32];
            if se.board[row][col] == 5 {
                bk = coord;
            } else if se.board[row][col] == 11 {
                wk = coord;
            }
        }
    }


    // Looping through all pieces and adding them to a list of moves

    let mut moves: Vec<Vec<i32>> = vec![];
    let mut moves_3d: Vec<Vec<Vec<i32>>> = vec![];


    for row in 0..8 {
        for col in 0..8 {
            match k {
                // check if the white king is in danger
                11 => {
                    match se.board[row][col] {
                        1 | 2 | 3 | 4 | 5 => {
                            let mut pos_moves = get_pos_moves(se, se.board[row][col] as i32, row as i32, col as i32);
                            moves_3d.push(pos_moves);
                        }
                        6 => {
                            let mut pos_moves: Vec<Vec<i32>> = vec![];
                            let mut mo: Vec<i32> = vec![];

                            mo = vec![row as i32 + 1, col as i32 + 1];
                            pos_moves.push(mo);
                            moves_3d.push(pos_moves.clone());

                            mo = vec![row as i32 + 1, col as i32 - 1];
                            pos_moves.push(mo);
                            moves_3d.push(pos_moves);
                        }
                        _ => {}
                    }
                }
                //Check if the black king is in danger
                5 => {
                    match se.board[row][col] {
                        7 | 8 | 9 | 10 | 11 => {
                            let mut pos_moves = get_pos_moves(se, se.board[row][col] as i32, row as i32, col as i32);
                            moves_3d.push(pos_moves);
                        }
                        12 => {
                            let mut pos_moves: Vec<Vec<i32>> = vec![];
                            let mut mo: Vec<i32> = vec![];

                            mo = vec![row as i32 - 1, col as i32 + 1];
                            pos_moves.push(mo);
                            moves_3d.push(pos_moves.clone());

                            mo = vec![row as i32 - 1, col as i32 - 1];
                            pos_moves.push(mo);
                            moves_3d.push(pos_moves);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }


    for vec in moves_3d {
        for scalar in vec {
            moves.push(scalar);
        }
    }


    if se.debug == true {
        write!(se.stdout,
               "{}{:?}",
               termion::cursor::Goto(1, 1),
               moves)
            .unwrap();

        write!(se.stdout,
               "{}{:?}",
               termion::cursor::Goto(10, 15),
               wk)
            .unwrap();

        write!(se.stdout,
               "{}{:?}",
               termion::cursor::Goto(10, 16),
               bk)
            .unwrap();
    }
    match k {
        5 => {
            if moves.contains(&bk) {
                se.game_state = GameState::BlackCheck;
                return true
            }

        }
        11 => {
            if moves.contains(&wk) {
                se.game_state = GameState::WhiteCheck;
                return true
            }
        }
        _ => {}
    }

    return false

}
fn get_pos_moves(se: &mut Game, piece: i32, row: i32, col: i32) -> Vec<Vec<i32>> {

    let mut moves: Vec<Vec<i32>> = vec![];

    match piece {
        1 | 7 => {

            for r in 1..8 {
                let mut pos_move_d = vec![];

                if row + r >= 8 {
                    break
                }

                if se.board[row as usize + r as usize][col as usize] != 0 {
                    pos_move_d = vec![row + r as i32, col];
                    moves.push(pos_move_d);
                    break
                }

                pos_move_d = vec![row + r as i32, col];
                moves.push(pos_move_d)
            }

            for r in 1..8 {
                let mut pos_move_u = vec![];

                if row - r < 0 {
                    break
                }

                if se.board[row as usize - r as usize][col as usize] != 0 {
                    pos_move_u = vec![row - r as i32, col];
                    moves.push(pos_move_u);
                    break
                }

                pos_move_u = vec![row - r as i32, col];
                moves.push(pos_move_u)
            }

            for r in 1..8 {
                let mut pos_move_l = vec![];

                if col - r < 0 {
                    break
                }

                if se.board[row as usize][col as usize - r as usize] != 0 {
                    pos_move_l = vec![row, col - r as i32];
                    moves.push(pos_move_l);
                    break
                }

                pos_move_l = vec![row, col - r as i32];
                moves.push(pos_move_l)
            }

            for r in 1..8 {
                let mut pos_move_r = vec![];

                if col + r >= 8 {
                    break
                }

                if se.board[row as usize][col as usize + r as usize] != 0 {
                    pos_move_r = vec![row, col + r as i32];
                    moves.push(pos_move_r);
                    break
                }

                pos_move_r = vec![row, col + r as i32];
                moves.push(pos_move_r)
            }

        }
        2 | 8 => {

            let mut pos1 = vec![row - 1, col + 2];
            let mut pos2 = vec![row + 1, col + 2];
            let mut pos3 = vec![row - 1, col - 2];
            let mut pos4 = vec![row + 1, col - 2];


            let mut pos5 = vec![row - 2, col + 1];
            let mut pos6 = vec![row - 2, col - 1];
            let mut pos7 = vec![row + 2, col + 1];
            let mut pos8 = vec![row + 2, col - 1];


            moves.push(pos1);
            moves.push(pos2);
            moves.push(pos3);
            moves.push(pos4);
            moves.push(pos5);
            moves.push(pos6);
            moves.push(pos7);
            moves.push(pos8);

        }
        3 | 9 => {

            // Up-Left

            for b in 1..8 {
                let mut pos_move_ul = vec![];

                if row - b < 0 || col - b < 0 {
                    break
                }

                if se.board[row as usize - b as usize][col as usize - b as usize] != 0 {
                    pos_move_ul = vec![row - b, col - b];
                    moves.push(pos_move_ul);
                    break
                }

                pos_move_ul = vec![row - b, col - b];
                moves.push(pos_move_ul);
            }

            // Up-Right

            for b in 1..8 {
                let mut pos_move_ur = vec![];

                if row - b < 0 || col + b >= 8 {
                    break
                }

                if se.board[row as usize - b as usize][col as usize + b as usize] != 0 {
                    pos_move_ur = vec![row - b, col + b];
                    moves.push(pos_move_ur);
                    break
                }

                pos_move_ur = vec![row - b, col + b];
                moves.push(pos_move_ur)

            }

            // Down-Left

            for b in 1..8 {
                let mut pos_move_dl = vec![];

                if row + b >= 8 || col - b < 0 {
                    break
                }

                if se.board[row as usize + b as usize][col as usize - b as usize] != 0 {
                    pos_move_dl = vec![row + b, col - b];
                    moves.push(pos_move_dl);
                    break

                }

                pos_move_dl = vec![row + b, col - b];
                moves.push(pos_move_dl);
            }

            for b in 1..8 {
                let mut pos_move_dr = vec![];

                if row + b >= 8 || col + b >= 8 {
                    break
                }

                if se.board[row as usize + b as usize][col as usize + b as usize] != 0 {
                    pos_move_dr = vec![row + b, col + b];
                    moves.push(pos_move_dr);
                    break
                }

                pos_move_dr = vec![row + b, col + b];
                moves.push(pos_move_dr);

            }

        }
        4 | 10 => {
            // Up-Left

            for b in 1..8 {
                let mut pos_move_ul = vec![];

                if row - b < 0 || col - b < 0 {
                    break
                }

                if se.board[row as usize - b as usize][col as usize - b as usize] != 0 {
                    pos_move_ul = vec![row - b, col - b];
                    moves.push(pos_move_ul);
                    break
                }

                pos_move_ul = vec![row - b, col - b];
                moves.push(pos_move_ul);
            }

            // Up-Right

            for b in 1..8 {
                let mut pos_move_ur = vec![];

                if row - b < 0 || col + b >= 8 {
                    break
                }

                if se.board[row as usize - b as usize][col as usize + b as usize] != 0 {
                    pos_move_ur = vec![row - b, col + b];
                    moves.push(pos_move_ur);
                    break
                }

                pos_move_ur = vec![row - b, col + b];
                moves.push(pos_move_ur)

            }

            // Down-Left

            for b in 1..8 {
                let mut pos_move_dl = vec![];

                if row + b >= 8 || col - b < 0 {
                    break
                }

                if se.board[row as usize + b as usize][col as usize - b as usize] != 0 {
                    pos_move_dl = vec![row + b, col - b];
                    moves.push(pos_move_dl);
                    break

                }

                pos_move_dl = vec![row + b, col - b];
                moves.push(pos_move_dl);
            }

            for b in 1..8 {
                let mut pos_move_dr = vec![];

                if row + b >= 8 || col + b >= 8 {
                    break
                }

                if se.board[row as usize + b as usize][col as usize + b as usize] != 0 {
                    pos_move_dr = vec![row + b, col + b];
                    moves.push(pos_move_dr);
                    break
                }

                pos_move_dr = vec![row + b, col + b];
                moves.push(pos_move_dr);

            }

            for r in 1..8 {
                let mut pos_move_d = vec![];

                if row + r >= 8 {
                    break
                }

                if se.board[row as usize + r as usize][col as usize] != 0 {
                    pos_move_d = vec![row + r as i32, col];
                    moves.push(pos_move_d);
                    break
                }

                pos_move_d = vec![row + r as i32, col];
                moves.push(pos_move_d)
            }

            for r in 1..8 {
                let mut pos_move_u = vec![];

                if row - r < 0 {
                    break
                }

                if se.board[row as usize - r as usize][col as usize] != 0 {
                    pos_move_u = vec![row - r as i32, col];
                    moves.push(pos_move_u);
                    break
                }

                pos_move_u = vec![row - r as i32, col];
                moves.push(pos_move_u)
            }

            for r in 1..8 {
                let mut pos_move_l = vec![];

                if col - r < 0 {
                    break
                }

                if se.board[row as usize][col as usize - r as usize] != 0 {
                    pos_move_l = vec![row, col - r as i32];
                    moves.push(pos_move_l);
                    break
                }

                pos_move_l = vec![row, col - r as i32];
                moves.push(pos_move_l)
            }

            for r in 1..8 {
                let mut pos_move_r = vec![];

                if col + r >= 8 {
                    break
                }

                if se.board[row as usize][col as usize + r as usize] != 0 {
                    pos_move_r = vec![row, col + r as i32];
                    moves.push(pos_move_r);
                    break
                }

                pos_move_r = vec![row, col + r as i32];
                moves.push(pos_move_r)
            }




        }
        5 | 11 => {

            let mut pos1 = vec![row - 1, col];
            let mut pos3 = vec![row - 1, col + 1];
            let mut pos2 = vec![row - 1, col - 1];

            let mut pos5 = vec![row, col - 1];
            let mut pos4 = vec![row, col + 1];

            let mut pos6 = vec![row + 1, col ];
            let mut pos7 = vec![row + 1, col - 1];
            let mut pos8 = vec![row + 1, col + 1];

            moves.push(pos1);
            moves.push(pos2);
            moves.push(pos3);
            moves.push(pos4);
            moves.push(pos5);
            moves.push(pos6);
            moves.push(pos7);
            moves.push(pos8);

        }
        6 => {
            let mut pos_moves_p = vec![];

            if row == 7 {
                return moves
            }

            if se.board[row as usize + 1][col as usize] == 0 {
                pos_moves_p = vec![row + 1, col];
                moves.push(pos_moves_p);
            }

            if row == 1 && se.board[row as usize + 1][col as usize] == 0 && se.board[row as usize + 2][col as usize] == 0 {
                pos_moves_p = vec![row + 2, col];
                moves.push(pos_moves_p);
            }

            if col != 0 {
                match se.board[row as usize - 1][col as usize - 1] {
                    1 | 2 | 3 | 4 | 5 | 6 => {
                        pos_moves_p = vec![row - 1, col - 1];
                        moves.push(pos_moves_p);
                    }
                    _ => {}
                }

            }

            if col != 7 {
                match se.board[row as usize - 1][col as usize + 1] {
                    1 | 2 | 3 | 4 | 5 | 6 => {
                        pos_moves_p = vec![row - 1, col + 1];
                        moves.push(pos_moves_p);
                    }
                    _ => {}
                }
            }
        }
        12 => {
            let mut pos_moves_p = vec![];

            if row == 0 {
                return moves
            }

            if se.board[row as usize - 1][col as usize] == 0 {
                pos_moves_p = vec![row - 1, col];
                moves.push(pos_moves_p);
            }

            if row == 6 && se.board[row as usize - 1][col as usize] == 0 && se.board[row as usize - 2][col as usize] == 0 {
                pos_moves_p = vec![row - 2, col];
                moves.push(pos_moves_p);
            }

            if col != 0 {
                match se.board[row as usize - 1][col as usize - 1] {
                    1 | 2 | 3 | 4 | 5 | 6 => {
                        pos_moves_p = vec![row - 1, col - 1];
                        moves.push(pos_moves_p);
                    }
                    _ => {}
                }

            }

            if col != 7 {
                match se.board[row as usize - 1][col as usize + 1] {
                    1 | 2 | 3 | 4 | 5 | 6 => {
                        pos_moves_p = vec![row - 1, col + 1];
                        moves.push(pos_moves_p);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    return moves
}
fn mate(se: &mut Game, k: i32) -> bool {
    let mut board = se.board;
    let mut moves: Vec<Vec<i32>> = vec![];

    for row in 0..8 {
        for col in 0..8 {
            match k {
                5 => {
                    match se.board[row][col] {
                        1 | 2 | 3 | 4 | 5 | 6 => {
                            let mo = get_pos_moves(se, se.board[row][col] as i32, row as i32, col as i32);
                            for m in mo {
                                let (mut x, mut y) = (m[0], m[1]);
                                let mut coord = vec![row as i32, col as i32, x, y];
                                moves.push(coord);
                            }
                        }
                        _ => {}
                    }
                }
                11 => {
                    match se.board[row][col] {
                        7 | 8 | 9 | 10 | 11 | 12 => {
                            let mo = get_pos_moves(se, se.board[row][col] as i32, row as i32, col as i32);
                            for m in mo {
                                let (mut x, mut y) = (m[0], m[1]);
                                let mut coord = vec![row as i32, col as i32, x, y];
                                moves.push(coord);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    let mut valid_moves1 = vec![];
    let mut valid_moves2 = vec![];

    for m in &moves {
        if m[0] >= 0 && m[0] <= 7 && m[1] >= 0 && m[1] <= 7 && m[2] >= 0 && m[2] <= 7 && m[3] >= 0 && m[3] <= 7 {
            valid_moves1.push(m);
        }

    }

    for m in valid_moves1 {
        match se.board[m[0] as usize][m[1] as usize] {
            1 | 2 | 3 | 4 | 5 | 6 => {
                match se.board[m[2] as usize][m[3] as usize] {
                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                        valid_moves2.push(m);
                    }
                    _ => {}
                }
            }
            7 | 8 | 9 | 10 | 11 | 12 => {
                match se.board[m[2] as usize][m[3] as usize] {
                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                        valid_moves2.push(m);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

    }


    for m in &valid_moves2 {
        let (mut x1,mut y1, mut x2, mut y2) = (m[0], m[1], m[2], m[3]);
        se.board[x2 as usize][y2 as usize] = se.board[x1 as usize][y1 as usize] as usize;
        se.board[x1 as usize][y1 as usize] = 0;

        if check(se, k) == false {
            se.board = board;
            draw(1, 10, "NO CHECKMATE".to_string());
            return false
        }

        se.board = board;
    }


    if se.debug == true {
        write!(se.stdout,
               "{}{:?}",
               termion::cursor::Goto(1, 45),
               valid_moves2.clone())
            .unwrap();
    }

    write!(se.stdout,
           "{}{:?}",
           termion::cursor::Goto(1, 45),
           valid_moves2.clone())
        .unwrap();

    return true
}
fn get_best_move(fen_board: String) -> String {
    let mut best_move = "".to_string();

    let options = ScriptOptions::new();
    let args = vec![];


    let cmd = format!(r#"
         stockfish << EOF
         uci
         position {}
         go movetime 6000
         ucinewgame
         EOF
         "#, fen_board);

    // ERROR: get_fen_bo GL!

    let (code, output, error) = run_script::run(
        &cmd as &str,
        &args,
        &options,
    )
        .unwrap();

    let mut best_moves_line = "".to_string();
    let bm = "bestmove";
    let op_max = output.len();


    //println!("{:?}", output);

    if output.contains(bm) {
        best_move = "".to_string();
        let bm_index = output.find(bm).unwrap();
        for index in bm_index..op_max {
            best_moves_line += &output.chars().nth(index).unwrap().to_string();
        }

        for b in 9..14 {
            best_move += &best_moves_line.chars().nth(b).unwrap().to_string();
        }

    }




    return best_move
}
fn board_to_fen(se: &mut Game) -> String {

    let mut fen = "fen ".to_string();
    let mut zeros_counter = 0;

    for row in 0..8 {
        if zeros_counter > 0 {
            fen += &zeros_counter.to_string() as &str;
        }
        zeros_counter = 0;
        if row != 0 {
            fen += "/";
        }
        for col in 0..8 {
            match se.board[row][col] {
                1 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "r";
                }
                2 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "n";
                }
                3 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "b";
                }
                4 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "q";
                }
                5 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "k";
                }
                6 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "p";
                }
                7 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "R";
                }
                8 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "N";
                }
                9 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "B";
                }
                10 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "Q";
                }
                11 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "K";
                }
                12 => {
                    if zeros_counter != 0 {
                        fen += &zeros_counter.to_string() as &str;
                        zeros_counter = 0;
                    }
                    fen += "P";
                }
                0 => {
                    zeros_counter += 1;
                }
                _ => {}
            }
        }
    }

    fen += " ";

    match se.round {
        Round::White => {
            fen += "w";
        }
        Round::Black => {
            fen += "b";
        }
        _ => {}
    }

    fen += " ";

    if se.wk_moved == false {

        if se.rwr_moved == false {
            fen += "K";
        }

        if se.lwr_moved == false {
            fen += "Q";
        }

    }

    if se.bk_moved == false {

        if se.rbr_moved == false {
            fen += "k";
        }

        if se.lbr_moved == false {
            fen += "q";
        }

    }

    if se.wk_moved && se.bk_moved && se.lwr_moved && se.rwr_moved && se.lbr_moved && se.rbr_moved {
        fen += "-"
    }

    fen += " ";

    if se.last_en_passant != vec![] {
        let pos = &se.last_en_passant;
        let last_en_passant_square = bo_cmd_to_fen_cmd(pos);
        fen += &last_en_passant_square as &str;
    } else if se.last_en_passant == vec![] {
        fen += "-"
    }

    fen += " ";

    fen += "-";

    fen += " ";

    fen += &se.move_count.trunc().to_string() as &str;



    return fen
}
fn bo_cmd_to_fen_cmd(c: &Vec<i32>) -> String {
    let mut cmd: String = "".to_string();

    // 5 0

    match c[1]{
        0 => {
            cmd += "a";
        }
        1 => {
            cmd += "b";
        }
        2 => {
            cmd += "c";
        }
        3 => {
            cmd += "d";
        }
        4 => {
            cmd += "e";
        }
        5 => {
            cmd += "f";
        }
        6 => {
            cmd += "g";
        }
        7 => {
            cmd += "h";
        }
        _ => {}
    }

    match c[0] {
        0 => {
            cmd += "8";
        }
        1 => {
            cmd += "7";
        }
        2 => {
            cmd += "6";
        }
        3 => {
            cmd += "5";
        }
        4 => {
            cmd += "4";
        }
        5 => {
            cmd += "3";
        }
        6 => {
            cmd += "2";
        }
        7 => {
            cmd += "1";
        }
        _ => {}
    }

    return cmd
}
fn fen_cmd_to_bo_cmd(c: String) -> Vec<i32> {
    let mut cmd: Vec<i32> = vec![];

    match c.chars().nth(1).unwrap() {
        '8' => {
            cmd.push(0);
        }
        '7' => {
            cmd.push(1);
        }
        '6' => {
            cmd.push(2);
        }
        '5' => {
            cmd.push(3);
        }
        '4' => {
            cmd.push(4);
        }
        '3' => {
            cmd.push(5);
        }
        '2' => {
            cmd.push(6);
        }
        '1' => {
            cmd.push(7);
        }
        _ => {}
    }

    match c.chars().nth(0).unwrap() {
        'a' => {
            cmd.push(0);
        }
        'b' => {
            cmd.push(1);
        }
        'c' => {
            cmd.push(2);
        }
        'd' => {
            cmd.push(3);
        }
        'e' => {
            cmd.push(4);
        }
        'f' => {
            cmd.push(5);
        }
        'g' => {
            cmd.push(6);
        }
        'h' => {
            cmd.push(7);
        }
        _ => {}
    }

    return cmd
}
fn init(se: &mut Game) {

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();


    write!(se.stdout,
           "{}{}{}",
           termion::clear::All,
           termion::cursor::Goto(1, 1),
           termion::cursor::Hide)
        .unwrap();



    default_setup(se);


    for c in stdin.keys() {
        write!(se.stdout,
               "{}{}",
               termion::cursor::Goto(1, 1),
               termion::clear::CurrentLine)
            .unwrap();

        match c.unwrap() {
            Key::Char('q') => break,
            Key::Char('r') => game(se),
            Key::Char('h') => {
                if se.game_started == false {
                    help_screen(se)
                }
            },
            Key::Char('s') => {
                if se.game_started == false {
                    start_screen(se)
                }
            },
            Key::Char('m') => {
                if se.game_started {
                    move_piece(se)
                }
            },
            Key::Char('d') => {
                if se.game_started == true {
                    if se.debug == true {
                        se.debug = false;
                        info_board(se);
                    } else if se.debug == false {
                        se.debug = true;
                        info_board(se);
                    }
                }
            }
            Key::Char('o') => {
                if se.game_started == false {
                    modes(se);
                }
            }
            Key::Char('n') => {
                if se.mode_screen == true {
                    if se.game_started == false {
                        if se.mode == Modes::PvP {
                            se.mode = Modes::AI;
                            modes(se);
                        } else if se.mode == Modes::AI {
                            se.mode = Modes::PvP;
                            modes(se);
                        }
                    }
                }
            }
            _ => {}
        }
        se.stdout.flush().unwrap();
    }

    write!(se.stdout, "{}", termion::cursor::Show).unwrap();
}

fn promoting_screen(se: &mut Game) -> i32 {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut piece = -1;

    let mut promo_screen = [
        "╔═══════════════════════════════════════╗",
        "║─────────PROMOTE PAWN──────────────────║",
        "║───────────────────────────────────────║",
        "║    ♜         ♞         ♝         ♛    ║",
        "║    |         |         |         |    ║",
        "║    R         N         B         Q    ║",
        "╚═══════════════════════════════════════╝"
    ];

    write!(se.stdout,
           "{}{}{}",
           termion::clear::All,
           termion::cursor::Goto(1, 1),
           termion::cursor::Hide)
        .unwrap();


    let mut array_counter = 20;

    se.stdout.flush().unwrap();


    for x in promo_screen {
        write!(se.stdout, "{}{}{}",
               termion::cursor::Goto(20, array_counter),
               x,
               termion::cursor::Hide).unwrap();
        se.stdout.flush().unwrap();
        array_counter += 1
    }

    for c in stdin.keys() {
        write!(se.stdout,
               "{}{}",
               termion::cursor::Goto(1, 1),
               termion::clear::CurrentLine)
            .unwrap();

        match c.unwrap() {
            Key::Char('R') | Key::Char('r' )=> {
                piece = 1;
                break;
            },
            Key::Char('N') | Key::Char('n') => {
                piece = 2;
                break;
            },
            Key::Char('B') | Key::Char('b') => {
                piece = 3;
                break;
            }
            Key::Char('Q') | Key::Char('q') => {
                piece = 4;
                break;
            }
            _ => {}
        }
    }

    write!(se.stdout,
           "{}{}{}",
           termion::clear::All,
           termion::cursor::Goto(1, 1),
           termion::cursor::Hide)
        .unwrap();

    se.stdout.flush().unwrap();

// HERE
    return piece
}

fn main(){

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut game: Game = Game {
        stdout: stdout,
        stdin: stdin,
        board: [[1, 0, 0, 0, 5, 0, 0, 1],
                [6, 6, 6, 6, 6, 6, 6, 6],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [12, 12, 12, 12, 12, 12, 12, 12],
                [7, 0, 0, 0, 11, 0, 0, 7]],
        game_state: GameState::Playing,
        round: Round::White,
        debug: false,
        wk_moved: false,
        bk_moved: false,
        lwr_moved: false,
        rwr_moved: false,
        lbr_moved: false,
        rbr_moved: false,
        white_captures: vec![],
        black_captures: vec![],
        move_info: MoveInfo::Null,
        mode: Modes::PvP,
        game_started: false,
        mode_screen: false,
        last_en_passant: vec![],
        move_count: 1.0,
    };

    let f = board_to_fen(&mut game);
    println!("{}", f);

    let b = get_best_move(f);
    println!("{}", b);


   // let f = get_best_move("'fen rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2'".to_string());
    init(&mut game);
    //println!("{}", f);
   // let fen = board_to_fen(&mut game);
  //  println!("{}", fen);
}

//TODO: AI: Captures, Rochades, Promotes
//TODO: Draws/Stalemates
//TODO: Fix long Rochades
