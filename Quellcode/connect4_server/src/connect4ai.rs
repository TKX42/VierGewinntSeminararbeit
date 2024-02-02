#![allow(dead_code)] // suppress weird clippy behaviour where used code is marked as unused

use std::collections::{BTreeMap, HashMap};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

pub const USER_PLAYER: u8 = 1;
pub const COMPUTER_PLAYER: u8 = 2;
pub const HEIGHT: usize = 6;
pub const WIDTH: usize = 7;

pub const MAX_SCORE: i64 = i64::MAX;
const MIN_SCORE: i64 = -MAX_SCORE;
const ZUGZWANG_SCORE: i64 = 100000000;
const THREAT_L4_SCORE: i64 = 10000;
const CENTRALITY_SCORE: usize = 1000;

pub struct Difficulty {
    calculation_depth: u8,
    zugzwang_evaluation: bool,
}

impl Difficulty {
    pub fn from_int(difficulty: u8) -> Difficulty {
        match difficulty {
            // Easy
            0 => Difficulty {
                calculation_depth: 4,
                zugzwang_evaluation: false,
            },

            // Medium
            1 => Difficulty {
                calculation_depth: 6,
                zugzwang_evaluation: true,
            },

            // Hard
            _ => Difficulty {
                calculation_depth: 8,
                zugzwang_evaluation: true,
            },
        }
    }
}

#[derive(PartialEq, Debug, Serialize)]
pub enum NextMoveResult {
    NextMove,
    ComputerWins,
    PlayerWins,
    Draw,
    None,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Field {
    pub x: u8,
    pub y: u8,
}

impl Field {
    pub fn new(x: u8, y: u8) -> Field {
        Field { x, y }
    }
}

#[derive(Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct GameBoard {
    pub(crate) grid: [[u8; WIDTH]; HEIGHT],
}

impl GameBoard {
    pub fn new() -> GameBoard {
        GameBoard {
            grid: [[0u8; WIDTH]; HEIGHT],
        }
    }

    // positive x: left to right; positive y: high to low   (eg. 0,0 -> top left; 6,5 -> bottom right)
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.grid[y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: u8) {
        self.grid[y][x] = value;
    }

    fn height(&self) -> usize {
        WIDTH
    }

    pub fn from(grid: [[u8; WIDTH]; HEIGHT]) -> GameBoard {
        GameBoard { grid }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Zugzwang {
    fulfilment_position: Field,
    even: bool,
    player: u8,
}

impl Zugzwang {
    pub fn new(fulfilment_position: Field, even: bool, player: u8) -> Zugzwang {
        Zugzwang {
            fulfilment_position,
            even,
            player,
        }
    }

    pub fn create(fulfilment_position: Field, player: u8) -> Zugzwang {
        Zugzwang {
            fulfilment_position,
            even: fulfilment_position.y % 2 == 0,
            player,
        }
    }
}

/* gibt zurück bei übergebener Spielstellung:
    - den besten Zug für de Computer
    - ob mit diesem Zug der Sieg für einen der beiden Spieler einher geht
*/
pub fn next_move(
    game_board: &mut GameBoard,
    computer_started: bool,
    difficulty: &Difficulty,
) -> (Option<Field>, i64, NextMoveResult) {
    let mut evaluation_cache: HashMap<GameBoard, i64> = HashMap::new();

    let (mut field, val) = max(
        difficulty.calculation_depth,
        MIN_SCORE,
        MAX_SCORE,
        game_board,
        &mut evaluation_cache,
        computer_started,
        difficulty,
    );
    let mut next_move_result = NextMoveResult::NextMove;

    // wenn ein Sieg für den Gegner bereits entschieden ist, spielt der Computer ein zufälliges Feld
    let free_fields = available_fields(game_board);
    if field.is_none() && !free_fields.is_empty() {
        field = Some(*free_fields.choose(&mut rand::thread_rng()).unwrap());
    }

    if free_fields.is_empty() && val != MIN_SCORE {
        return (None, 0, NextMoveResult::Draw);
    }

    game_board.set(field.unwrap().x as usize, field.unwrap().y as usize, 2);
    if check_for_row(&game_board.grid, COMPUTER_PLAYER, 4).0 {
        next_move_result = NextMoveResult::ComputerWins;
    } else if check_for_row(&game_board.grid, USER_PLAYER, 4).0 {
        next_move_result = NextMoveResult::PlayerWins;
    }

    (field, val, next_move_result)
}

fn max(
    depth: u8,
    alpha: i64,
    beta: i64,
    game_board_variation: &mut GameBoard,
    evaluation_cache: &mut HashMap<GameBoard, i64>,
    player_started: bool,
    difficulty: &Difficulty,
) -> (Option<Field>, i64) {
    let mut result = None;
    let possible_moves = available_fields(game_board_variation); // Liste aller möglichen Züge

    /* breche die Rekursion ab und berechne den Score der aktuellen Spielstellung,
    wenn die maximale Tiefe erreicht ist, oder einer der beiden Spieler das Spiel gewonnen hat
    */
    if depth == 0
        || possible_moves.is_empty()
        || check_for_row(&game_board_variation.grid, COMPUTER_PLAYER, 4).0
        || check_for_row(&game_board_variation.grid, USER_PLAYER, 4).0
    {
        return (
            None,
            evaluation(
                game_board_variation,
                evaluation_cache,
                COMPUTER_PLAYER,
                player_started,
                difficulty.zugzwang_evaluation,
            ),
        );
    }

    // der Score des besten Zugs für den maximierenden Spieler (Computer)
    let mut max_val = alpha;

    for possible_move in possible_moves {
        game_board_variation.set(
            possible_move.x as usize,
            possible_move.y as usize,
            COMPUTER_PLAYER,
        ); // führe Zug aus

        let val = min(
            depth - 1,
            max_val,
            beta,
            game_board_variation,
            evaluation_cache,
            player_started,
            difficulty,
        )
        .1;

        game_board_variation.set(possible_move.x as usize, possible_move.y as usize, 0); // mache Zug rückgängig

        // ein besserer Zug wurde gefunden
        if val > max_val {
            max_val = val;

            // auf höchster Ebene ist der beste gefundene Zug der, der am Ende zurückgegeben wird
            if depth == difficulty.calculation_depth {
                result = Some(possible_move);
            }

            // Alpha-Beta-Pruning
            if max_val >= beta {
                break;
            }
        }
    }

    // gib den maximalen Zug-Score für die aktuelle Ebene zurück und auf der höchsten Ebene ebenfalls den dazugehörigen Zug
    (result, max_val)
}

fn min(
    depth: u8,
    alpha: i64,
    beta: i64,
    game_board_variation: &mut GameBoard,
    evaluation_cache: &mut HashMap<GameBoard, i64>,
    player_started: bool,
    difficulty: &Difficulty,
) -> (Option<Field>, i64) {
    let possible_moves = available_fields(game_board_variation); // Liste aller möglichen Züge

    /* breche die Rekursion ab und berechne den Score der aktuellen Spielstellung,
    wenn die maximale Tiefe erreicht ist, oder einer der beiden Spieler das Spiel gewonnen hat
    */
    if depth == 0
        || possible_moves.is_empty()
        || check_for_row(&game_board_variation.grid, COMPUTER_PLAYER, 4).0
        || check_for_row(&game_board_variation.grid, USER_PLAYER, 4).0
    {
        return (
            None,
            evaluation(
                game_board_variation,
                evaluation_cache,
                COMPUTER_PLAYER,
                player_started,
                difficulty.zugzwang_evaluation,
            ),
        );
    }

    // der Score des besten Zugs für den minimierenden Spieler (Gegner des Computers)
    let mut min_val = beta;

    for possible_move in possible_moves {
        game_board_variation.set(
            possible_move.x as usize,
            possible_move.y as usize,
            USER_PLAYER,
        ); // führe Zug aus
        let val = max(
            depth - 1,
            alpha,
            min_val,
            game_board_variation,
            evaluation_cache,
            player_started,
            difficulty,
        )
        .1;
        game_board_variation.set(possible_move.x as usize, possible_move.y as usize, 0); // mache Zug rückgängig

        // ein besserer Zug wurde gefunden
        if val < min_val {
            min_val = val;

            // Alpha-Beta-Pruning
            if min_val <= alpha {
                break;
            }
        }
    }

    // gib den minimalen Zug-Score für die aktuelle Ebene zurück
    (None, min_val)
}

// Bewertet die übergebene Spielposition aus Sicht des Computers mit Einbezug gegnerischer Felder
pub fn evaluation(
    game_board_variation: &GameBoard,
    evaluation_cache: &mut HashMap<GameBoard, i64>,
    player: u8,
    player_started: bool,
    zugzwang_evaluation: bool,
) -> i64 {
    // wenn ein score für diese Spielstellung bereits berechnet wurde, gib diesen zurück und berechne ihn nicht neu
    match evaluation_cache.get(game_board_variation) {
        None => {}
        Some(ev) => {
            return *ev;
        }
    }

    // Liste aller Zugzwänge
    let mut zugzwang_list: Vec<Zugzwang> = Vec::new();

    let max_ev = evaluate_game_position(game_board_variation, player, &mut zugzwang_list);
    let min_ev = evaluate_game_position(
        game_board_variation,
        other_player(player),
        &mut zugzwang_list,
    );

    // catch possible integer overflow
    if min_ev == MAX_SCORE {
        return -MAX_SCORE;
    }

    if max_ev == MAX_SCORE {
        return MAX_SCORE;
    }

    let mut result = max_ev - min_ev;

    // berechne die Bewertung der Zugzwänge für den Spieler
    if zugzwang_evaluation {
        result += evaluate_zugzwang_positions(zugzwang_list, player, player_started) as i64
            * ZUGZWANG_SCORE;
    }

    // füge den berechneten Score in den Cache ein
    evaluation_cache.insert(game_board_variation.clone(), result);
    result
}

pub fn evaluate_game_position(
    game_board_variation: &GameBoard,
    player: u8,
    zugzwang_list: &mut Vec<Zugzwang>,
) -> i64 {
    if check_for_row(&game_board_variation.grid, player, 4).0 {
        return MAX_SCORE;
    }

    let mut result: i64 = 0;

    // Bedrohungen inklusive Zugzwängen
    result +=
        evaluate_threats(&game_board_variation.grid, player, 4, zugzwang_list) * THREAT_L4_SCORE;

    // bewerte Feldpositionen: je mittiger desto besser
    result += evaluate_centrality(game_board_variation, player);

    result
}

fn evaluate_centrality(game_board_variation: &GameBoard, player: u8) -> i64 {
    let mut result = 0;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if game_board_variation.get(x, y) == player {
                result += evaluate_field_position(x) * CENTRALITY_SCORE;
            }
        }
    }
    result as i64
}

// bevorzuge Felder die in der Mitte legen
pub fn evaluate_field_position(x: usize) -> usize {
    let center_of_field: usize = WIDTH / 2;
    if x <= center_of_field {
        x
    } else {
        WIDTH - 1 - x
    }
}

pub fn available_fields(game_board_variation: &GameBoard) -> Vec<Field> {
    let mut available_fields = Vec::new();
    for (y, column) in game_board_variation.grid.iter().enumerate() {
        for (x, field) in column.iter().enumerate() {
            if *field == 0 && field_has_ground(game_board_variation, x, y) {
                available_fields.push(Field::new(x as u8, y as u8)) // field is free and has a ground
            }
        }
    }
    available_fields
}

fn field_has_ground(game_board_variation: &GameBoard, x: usize, y: usize) -> bool {
    if y + 1 > HEIGHT {
        return false;
    }
    y + 1 == HEIGHT || game_board_variation.get(x, y + 1) != 0
}

/*
   Berechnet die Anzahl an threats und fügt erkannte Zugzwänge in die übergebene Liste ein
*/
pub fn evaluate_threats(
    grid: &[[u8; WIDTH]; HEIGHT],
    player: u8,
    length: usize,
    zugzwang_list: &mut Vec<Zugzwang>,
) -> i64 {
    let mut count = 0;

    let mut diagonal_pattern_ends: Vec<Field> = Vec::new();
    let mut diagonal_mirrored_pattern_ends: Vec<Field> = Vec::new();

    for y in 0..HEIGHT {
        let mut x = 0;
        while x < WIDTH {
            let horizontal = check_sequence_horizontal(grid, player, length, &mut x, y);
            count += horizontal.0 as i64;
            if let Some(horizontal_zugzwang) = horizontal.1 {
                zugzwang_list.push(horizontal_zugzwang);
            }

            let diagonal =
                check_sequence_diagonal(grid, player, length, x, y, &mut diagonal_pattern_ends);
            count += diagonal.0 as i64;
            if let Some(diagonal_zugzwang) = diagonal.1 {
                zugzwang_list.push(diagonal_zugzwang);
            }

            x += 1;
        }
    }

    for x in 0..WIDTH {
        let mut y = 0;
        while y < HEIGHT {
            let diagonal_mirrored = check_sequence_diagonal_mirrored(
                grid,
                player,
                length,
                x,
                y,
                &mut diagonal_mirrored_pattern_ends,
            );
            count += diagonal_mirrored.0 as i64;
            if let Some(diagonal_mirrored_zugzwang) = diagonal_mirrored.1 {
                zugzwang_list.push(diagonal_mirrored_zugzwang);
            }
            y += 1;
        }
    }

    count
}

/*
   gibt einen möglichen Zugzwang zurück, wenn das Feld unterhalb der Erfüllungsposition frei ist
*/
fn is_possible_zugzwang(grid: &[[u8; 7]; 6], player: u8, y: usize, x: usize) -> Option<Zugzwang> {
    if y + 1 < HEIGHT && grid[y + 1][x] == 0 {
        // Feld unter der Erfüllungsposition ist frei
        return Some(Zugzwang::create(Field::new(x as u8, y as u8), player));
    }

    None
}

/*
   gibt zurück, ob es eine diagonale Sequenz mit Länge an Position nach rechts unten aus gibt,
   sowie wenn es sich um eine mögliche Zugzwang-Bedrohung handelt dessen Position
*/
pub fn check_sequence_diagonal(
    grid: &[[u8; WIDTH]; HEIGHT],
    player: u8,
    length: usize,
    start_x: usize,
    start_y: usize,
    pattern_ends: &mut Vec<Field>,
) -> (bool, Option<Zugzwang>) {
    // bound check
    let mut end_x = start_x + length - 1;
    let mut end_y = start_y + length - 1;
    if end_x >= WIDTH || end_y >= HEIGHT {
        return (false, None);
    }

    for pattern_end in pattern_ends.iter() {
        let x_diff: isize = start_x as isize - pattern_end.x as isize;
        let y_diff: isize = start_y as isize - pattern_end.y as isize;
        if x_diff <= 0 && y_diff <= 0 && x_diff == y_diff {
            return (false, None);
        }
    }

    let mut zugzwang: Option<Zugzwang> = None;
    let mut wildcard = true;

    for i in 0..length {
        let x = start_x + i;
        let y = start_y + i;
        if grid[y][x] != player {
            if grid[y][x] == 0 && wildcard {
                // werte aus, ob es sich um einen valide Zugzwang-Bedrohung handelt
                zugzwang = is_possible_zugzwang(grid, player, y, x);
                wildcard = false;
                // allow detection of multiple zugzwangs
                end_x = x;
                end_y = y;
                continue;
            }
            return (false, None);
        }
    }

    pattern_ends.push(Field::new(end_x as u8, end_y as u8));

    (true, zugzwang)
}

pub fn check_sequence_diagonal_mirrored(
    grid: &[[u8; WIDTH]; HEIGHT],
    player: u8,
    length: usize,
    start_x: usize,
    start_y: usize,
    pattern_ends: &mut Vec<Field>,
) -> (bool, Option<Zugzwang>) {
    // bound check
    let mut end_x: isize = start_x as isize - length as isize + 1;
    let mut end_y: isize = start_y as isize + length as isize - 1;
    if end_x < 0 || end_y >= HEIGHT as isize {
        return (false, None);
    }

    for pattern_end in pattern_ends.iter() {
        let x_diff: isize = start_x as isize - pattern_end.x as isize;
        let y_diff: isize = start_y as isize - pattern_end.y as isize;
        if x_diff >= 0 && y_diff <= 0 && x_diff == -y_diff {
            return (false, None);
        }
    }

    let mut zugzwang: Option<Zugzwang> = None;
    let mut wildcard = true;

    for i in 0..length {
        let x = start_x - i;
        let y = start_y + i;
        if grid[y][x] != player {
            if grid[y][x] == 0 && wildcard {
                // werte aus, ob es sich um einen valide Zugzwang-Bedrohung handelt
                zugzwang = is_possible_zugzwang(grid, player, y, x);
                wildcard = false;
                // allow detection of multiple zugzwangs
                end_x = x as isize;
                end_y = y as isize;
                continue;
            }
            return (false, None);
        }
    }

    pattern_ends.push(Field::new(end_x as u8, end_y as u8));

    (true, zugzwang)
}

/*
   gibt zurück, ob es eine horizontale Sequenz mit Länge an Position nach rechts aus gibt,
   sowie wenn es sich um eine mögliche Zugzwang-Bedrohung handelt dessen Position
*/
pub fn check_sequence_horizontal(
    grid: &[[u8; WIDTH]; HEIGHT],
    player: u8,
    length: usize,
    start_x: &mut usize,
    y: usize,
) -> (bool, Option<Zugzwang>) {
    // bound check
    let end_x = *start_x + length - 1;
    if end_x >= WIDTH {
        return (false, None);
    }

    // Das pattern endet früher wenn es eine Wildcard gibt, damit mehrere Zugzwänge erkannt werden
    let mut pattern_end = length - 1;
    let mut wildcard = true;
    let mut zugzwang: Option<Zugzwang> = None;

    for i in 0..length {
        let x = *start_x + i;
        if grid[y][x] != player {
            if grid[y][x] == 0 && wildcard {
                // werte aus, ob es sich um einen valide Zugzwang-Bedrohung handelt
                zugzwang = is_possible_zugzwang(grid, player, y, x);
                wildcard = false;
                pattern_end = i;
                continue;
            }
            return (false, None);
        }
    }

    // avoid duplicates
    *start_x += pattern_end;
    (true, zugzwang)
}

pub fn check_for_row(grid: &[[u8; WIDTH]; HEIGHT], player: u8, length: usize) -> (bool, Field) {
    for (y, column) in grid.iter().enumerate() {
        for (x, _field) in column.iter().enumerate() {
            if check_sequence(grid, player, length, x, y, 1, 0) ||      // horizontal nach rechts
                check_sequence(grid, player, length, x, y, 0, 1) ||     // vertikal nach unten
                check_sequence(grid, player, length, x, y, 1, 1) ||     // diagonal unten links nach oben rechts
                check_sequence(grid, player, length, x, y, -1, 1)
            // diagonal unten rechts nach oben links (gespiegelt)
            {
                return (true, Field::new(x as u8, y as u8));
            }
        }
    }
    (false, Field::new(0, 0))
}

fn check_sequence(
    grid: &[[u8; WIDTH]; HEIGHT],
    player: u8,
    length: usize,
    start_x: usize,
    start_y: usize,
    step_x: isize,
    step_y: isize,
) -> bool {
    // außerhalb des Spielfelds
    let end_x: isize = start_x as isize + (length as isize - 1) * step_x;
    let end_y: isize = start_y as isize + (length as isize - 1) * step_y;

    if end_x >= WIDTH.try_into().unwrap()
        || end_y >= HEIGHT.try_into().unwrap()
        || end_x < 0
        || end_y < 0
    {
        return false;
    }

    for i in 0..length {
        let x = (start_x as isize + i as isize * step_x) as usize;
        let y = (start_y as isize + i as isize * step_y) as usize;

        if grid[y][x] != player {
            return false;
        }
    }

    true
}

pub fn other_player(player: u8) -> u8 {
    if player == COMPUTER_PLAYER {
        USER_PLAYER
    } else {
        COMPUTER_PLAYER
    }
}

/*
   Bewertet die übergebene Liste an Zugzwängen aus Sicht des Spielers mithilfe einer regelbasierten Simulation
   Gibt -1 bei Niederlage für den Spieler, 1 bei Sieg für ihn und 0 bei einem Unentschieden zurück
*/
pub fn evaluate_zugzwang_positions(
    zugzwang_list: Vec<Zugzwang>,
    player: u8,
    player_started: bool,
) -> i8 {
    let zugzwang_map = sort_zugzwang_list(zugzwang_list);
    let width = zugzwang_map.len();

    let mut board: Vec<Vec<u8>> = vec![vec![0; HEIGHT]; width]; // greater y means chip is higher

    let shared_zugzwang = 4;
    for (i, (_, zugzwang_column)) in zugzwang_map.iter().enumerate() {
        for zugzwang in zugzwang_column.iter() {
            // convert top->bottom y to bottom->top y
            let y = HEIGHT - 1 - zugzwang.fulfilment_position.y as usize;

            // Zugzwang beider Spieler auf gleicher Stelle
            if board[i][y] == other_player(zugzwang.player) {
                board[i][y] = shared_zugzwang;
            }

            board[i][y] = zugzwang.player;
        }
    }

    let mut highest_chip: Vec<usize> = vec![0; width];
    let mut player_at_turn = if player_started {
        player
    } else {
        other_player(player)
    };
    loop {
        let result = simulate_zugzwang_turn(
            &mut board,
            player_at_turn,
            &mut highest_chip,
            shared_zugzwang,
        );
        if result != 0 {
            // unentschieden
            if result == 3 {
                return 0;
            }

            return if player_at_turn == player {
                result
            } else {
                -result
            };
        }
        player_at_turn = other_player(player_at_turn);
    }
}

/*
   Assigns the best move according to given rules
   Important: Rules need to be sorted by importance!
   first_priority: if true, the first rule of which the condition is true will be used regardless of its score
*/
fn evaluate_turns(rules: Vec<(i8, bool, bool)>, best_move: &mut (usize, i8), x: usize) {
    for (score, condition, first_priority) in rules {
        if condition {
            if best_move.1 < score {
                best_move.0 = x;
                best_move.1 = score;
                return;
            }

            // break the loop if a rule with priority is found
            if first_priority {
                return;
            }
        }
    }
}

/*
   Simuliert regelbasiert den Zug des übergebenen Spielers
   Gibt 0 bei einem nicht-finalem Zug, -1 bei Niederlage für den Spieler, 1 bei Sieg für ihn und 3 bei einem Unentschieden zurück
*/
fn simulate_zugzwang_turn(
    board: &mut Vec<Vec<u8>>,
    player: u8,
    highest_chip: &mut [usize],
    shared_zugzwang: u8,
) -> i8 {
    let mut filled_columns = 0;
    let mut best_move: (usize, i8) = (0, -1); // (x, score of move)

    // Berechne den best-möglichen Zug nach den Regeln indem jede Spalte einmal durchgegangen wird
    for (x, column) in board.iter_mut().enumerate() {
        let y = highest_chip[x];
        if y >= HEIGHT {
            filled_columns += 1;
            continue;
        }

        // Erfülle eigenen Zugzwang
        if column[y] == player || column[y] == shared_zugzwang {
            return 1;
        }
        // Spiele nicht unter den Zugzwang des Gegners
        else if y + 1 < HEIGHT
            && (column[y + 1] == other_player(player) || column[y + 1] == shared_zugzwang)
        {
            continue;
        }

        evaluate_turns(
            vec![
                // Blocke Zugzwang des Gegners
                (7, column[y] == other_player(player), true),
                // Fülle eine Spalte auf
                (6, column_is_empty(column, y), false),
                // Decke eigenen ungeraden Zugzwang auf ((y + 2) da y unten bei 0 startet)
                (
                    2,
                    y + 1 < column.len() && column[y + 1] == player && (y + 2) % 2 != 0,
                    true,
                ),
                // Decke eigenen Zugzwang auf
                (1, y + 1 < column.len() && column[y + 1] == player, true),
                // Spiele 2 Felder unter eigenen ungeraden Zugzwang (erlange Zugzwang-Kontrolle)
                (
                    5,
                    y + 2 < column.len() && column[y + 2] == player && (y + 1) % 2 == 0,
                    false,
                ),
                // Spiele 2 Felder unter eigenen Zugzwang
                (4, y + 2 < column.len() && column[y + 2] == player, false),
                // Spiele 2 Felder unter gegnerischen / geteilten Zugzwang
                (
                    4,
                    y + 2 < column.len()
                        && (column[y + 2] == other_player(player)
                            || column[y + 2] == shared_zugzwang),
                    false,
                ),
                // 2 Die untersten 3 Felder sind frei
                (
                    3,
                    y + 3 < column.len()
                        && column[y] == 0
                        && column[y + 1] == 0
                        && column[y + 2] == 0,
                    false,
                ),
                // Spiele ein Feld, auf und über dem kein eigener Zugzwang liegt
                (
                    1,
                    y + 1 < column.len() && column[y] != player && column[y + 1] != player,
                    false,
                ),
                // Spiele beliebiges Feld
                (0, true, false),
            ],
            &mut best_move,
            x,
        );
    }

    // Alle Felder sind gefüllt und niemand konnte einen Zugzwang verwirklichen
    if filled_columns == board.len() {
        return 3;
    }

    if best_move == (0, -1) {
        -1
    } else {
        /*println!(
            "{player} played {}/{} with score {}",
            best_move.0, highest_chip[best_move.0], best_move.1
        );

         */
        highest_chip[best_move.0] += 1;
        0
    }
}

// checks if the column is empty y upwards
fn column_is_empty(column: &[u8], y: usize) -> bool {
    for i in y..column.len() {
        if column[i] != 0 {
            return false;
        }
    }

    true
}

/*
   Sortiert die übergebene Liste an Zugzwängen in eine Map aus Spalte und den in dieser Spalte vorgefundenen Zugzwängen
*/
pub fn sort_zugzwang_list(zugzwang_list: Vec<Zugzwang>) -> BTreeMap<u8, Vec<Zugzwang>> {
    let mut zugzwang_map: BTreeMap<u8, Vec<Zugzwang>> = BTreeMap::new();

    zugzwang_list.into_iter().for_each(|zugzwang| {
        zugzwang_map
            .entry(zugzwang.fulfilment_position.x)
            .or_default()
            .push(zugzwang);
    });

    zugzwang_map
}
