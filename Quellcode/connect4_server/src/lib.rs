mod connect4ai;

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use crate::connect4ai::NextMoveResult::NextMove;
    use crate::connect4ai::{
        available_fields, check_for_row, check_sequence_diagonal, check_sequence_diagonal_mirrored,
        check_sequence_horizontal, evaluate_field_position, evaluate_game_position,
        evaluate_threats, evaluate_zugzwang_positions, evaluation, next_move, other_player,
        sort_zugzwang_list, Difficulty, Field, GameBoard, Zugzwang, COMPUTER_PLAYER, MAX_SCORE,
        USER_PLAYER,
    };

    /*
       EMPTY GRID TEMPLATE
       let grid: [[u8; 7]; 6] = [
           [0, 0, 0, 0, 0, 0, 0],
           [0, 0, 0, 0, 0, 0, 0],
           [0, 0, 0, 0, 0, 0, 0],
           [0, 0, 0, 0, 0, 0, 0],
           [0, 0, 0, 0, 0, 0, 0],
           [0, 0, 0, 0, 0, 0, 0],
       ];
    */

    #[test]
    fn game_board_set() {
        let mut game_board = GameBoard::new();
        game_board.set(0, 0, 1);
        game_board.set(1, 1, 2);
        game_board.set(6, 5, 2);
        assert_eq!(1, game_board.get(0, 0));
        assert_eq!(2, game_board.get(1, 1));
        assert_eq!(2, game_board.get(6, 5));
    }

    #[test]
    fn available_fields_test() {
        let mut game_board = GameBoard::new();
        game_board.set(0, 5, 1);
        game_board.set(1, 4, 2);
        game_board.set(6, 5, 2);

        let available_fields = available_fields(&game_board);

        let expected_fields = vec![
            Field { x: 1, y: 3 },
            Field { x: 0, y: 4 },
            Field { x: 6, y: 4 },
            Field { x: 1, y: 5 },
            Field { x: 2, y: 5 },
            Field { x: 3, y: 5 },
            Field { x: 4, y: 5 },
            Field { x: 5, y: 5 },
        ];

        assert_eq!(available_fields, expected_fields);
    }

    #[test]
    fn other_player_test() {
        assert_eq!(2, other_player(1));
        assert_eq!(1, other_player(2));
    }

    #[test]
    fn check_for_row_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 1, 1, 1, 1, 0, 0],
        ];

        assert_eq!((true, Field::new(1, 5)), check_for_row(&grid, 1, 4));

        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 1, 0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
            [0, 0, 0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
        ];

        assert_eq!((true, Field::new(1, 1)), check_for_row(&grid, 1, 4));

        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 0],
        ];

        assert_eq!((true, Field::new(6, 1)), check_for_row(&grid, 1, 4));

        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 2, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
            [0, 0, 1, 0, 0, 0, 0],
            [0, 1, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
        ];

        assert_eq!((true, Field::new(3, 1)), check_for_row(&grid, 1, 4));
    }

    #[test]
    fn test_field_evaluation() {
        let x = evaluate_field_position(0);
        assert_eq!(x, 0);

        let x = evaluate_field_position(1);
        assert_eq!(x, 1);

        let x = evaluate_field_position(2);
        assert_eq!(x, 2);

        let x = evaluate_field_position(3);
        assert_eq!(x, 3);

        let x = evaluate_field_position(4);
        assert_eq!(x, 2);

        let x = evaluate_field_position(5);
        assert_eq!(x, 1);

        let x = evaluate_field_position(6);
        assert_eq!(x, 0);
    }

    #[test]
    fn test_basic_dilemma() {
        let mut game_board = GameBoard::new();
        game_board.set(1, 5, USER_PLAYER);
        game_board.set(2, 5, USER_PLAYER);
        game_board.set(3, 5, USER_PLAYER);

        let next_move_result = next_move(&mut game_board, false, &Difficulty::from_int(3));

        assert!(next_move_result.0.is_some());
        assert_eq!(-MAX_SCORE, next_move_result.1);
        assert_eq!(NextMove, next_move_result.2);
    }

    #[test]
    fn test_basic_row_avert() {
        let mut game_board = GameBoard::new();
        game_board.set(0, 5, USER_PLAYER);
        game_board.set(1, 5, USER_PLAYER);
        game_board.set(2, 5, USER_PLAYER);

        let next_move = next_move(&mut game_board, false, &Difficulty::from_int(3))
            .0
            .unwrap();
        assert_eq!((3, 5), (next_move.x, next_move.y));
    }

    #[test]
    fn test_start_move() {
        let mut game_board = GameBoard::new();
        game_board.set(3, 5, USER_PLAYER);

        let next_move = next_move(&mut game_board, false, &Difficulty::from_int(3))
            .0
            .unwrap();
        assert_eq!((3, 4), (next_move.x, next_move.y));
    }

    #[test]
    fn performance_test() {
        let mut game_board = GameBoard::new();
        game_board.set(3, 5, USER_PLAYER);

        use std::time::Instant;
        let now = Instant::now();

        let next_move = next_move(&mut game_board, false, &Difficulty::from_int(3))
            .0
            .unwrap();

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        assert_eq!((3, 4), (next_move.x, next_move.y));
    }

    #[test]
    fn check_sequence_horizontal_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [2, 2, 0, 2, 1, 0, 0],
            [0, 0, 0, 2, 2, 2, 2],
            [0, 0, 0, 0, 0, 0, 0],
        ];

        let mut x = 0;
        assert!(check_sequence_horizontal(&grid, 2, 4, &mut x, 3).0);
        assert_eq!(2, x);
        x = 3;
        assert!(check_sequence_horizontal(&grid, 2, 4, &mut x, 4).0);
        assert_eq!(6, x);
    }

    #[test]
    fn check_sequence_horizontal_not_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [2, 2, 1, 2, 1, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
        ];

        let mut x = 0;
        assert!(!check_sequence_horizontal(&grid, 2, 4, &mut x, 3).0);
        assert_eq!(0, x);
    }

    #[test]
    fn evaluate_threats_test_horizontal() {
        let grid: [[u8; 7]; 6] = [
            [2, 1, 1, 2, 0, 2, 2], // 3-6
            [2, 2, 0, 2, 0, 0, 0], // 0-3
            [0, 0, 0, 0, 0, 0, 0], // -
            [0, 0, 0, 0, 0, 0, 0], // -
            [0, 0, 0, 0, 0, 0, 0], // -
            [0, 0, 2, 2, 2, 2, 0], // 2-5
        ];

        assert_eq!(4, evaluate_threats(&grid, 2, 4, &mut Vec::new()));
    }

    #[test]
    fn check_sequence_diagonal_simple_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 0, 0],
            [0, 2, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 1, 1, 2, 1, 0, 0],
        ];

        let mut pattern_ends: Vec<Field> = Vec::new();
        assert_eq!(
            true,
            check_sequence_diagonal(&grid, 2, 4, 0, 2, &mut pattern_ends).0
        );
        assert_eq!([Field::new(2, 4)], pattern_ends.as_slice());
    }

    #[test]
    fn check_sequence_diagonal_avoid_duplicates_test() {
        let grid: [[u8; 7]; 6] = [
            [2, 0, 0, 0, 0, 0, 0],
            [0, 2, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 0, 2, 0, 0],
            [0, 1, 1, 2, 1, 2, 0],
        ];

        assert_eq!(1, evaluate_threats(&grid, 2, 4, &mut Vec::new()));
    }

    #[test]
    fn check_sequence_diagonal_mirrored_avoid_duplicates_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 1, 0],
            [0, 0, 0, 0, 1, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
            [0, 0, 1, 0, 0, 0, 0],
            [0, 1, 0, 0, 0, 0, 0],
            [1, 1, 1, 2, 1, 0, 0],
        ];

        assert_eq!(1, evaluate_threats(&grid, 1, 4, &mut Vec::new()));
    }

    #[test]
    fn check_sequence_multiple_diagonals_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 1, 0],
            [0, 1, 0, 0, 1, 0, 0],
            [0, 0, 1, 1, 0, 0, 0],
            [0, 0, 1, 0, 1, 0, 0],
            [0, 1, 0, 0, 1, 1, 0],
            [0, 0, 0, 0, 0, 0, 0],
        ];

        assert_eq!(4, evaluate_threats(&grid, 1, 4, &mut Vec::new()));
    }

    #[test]
    fn check_sequence_bound_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 0, 0, 2, 0, 0],
            [0, 0, 0, 0, 0, 2, 0],
            [2, 0, 0, 0, 0, 0, 2],
            [2, 0, 0, 0, 0, 0, 0],
            [2, 0, 0, 0, 2, 2, 2],
        ];

        assert_eq!(2, evaluate_threats(&grid, 2, 4, &mut Vec::new()));
    }

    #[test]
    fn check_sequence_diagonal_mirrored_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 2, 0, 0, 0, 0],
            [0, 2, 0, 0, 0, 0, 0],
        ];

        assert!(check_sequence_diagonal_mirrored(&grid, 2, 4, 4, 2, &mut Vec::new()).0);
        assert_eq!(1, evaluate_threats(&grid, 2, 4, &mut Vec::new()));
    }

    #[test]
    fn threats_evaluation_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [0, 0, 2, 0, 0, 0, 0],
            [0, 2, 0, 0, 0, 0, 0],
        ];

        let mut game_board = GameBoard::from(grid);

        let ev_sequence = evaluation(
            &game_board,
            &mut HashMap::new(),
            COMPUTER_PLAYER,
            false,
            true,
        );

        game_board.set(4, 2, 1);

        // create zugzwang list and print it
        let mut zugzwang_list: Vec<Zugzwang> = Vec::new();
        evaluate_threats(&game_board.grid, 1, 4, &mut zugzwang_list);
        evaluate_threats(&game_board.grid, 2, 4, &mut zugzwang_list);

        let ev_sequence_averted = evaluation(
            &game_board,
            &mut HashMap::new(),
            COMPUTER_PLAYER,
            false,
            true,
        );
        assert_eq!(0, evaluate_threats(&game_board.grid, 2, 4, &mut Vec::new()));
        assert!(ev_sequence > ev_sequence_averted);
    }

    #[test]
    fn sort_zugzwang_list_test() {
        let zugzwang_list = vec![
            Zugzwang::new(Field::new(2, 3), false, 1),
            Zugzwang::new(Field::new(2, 4), false, 2),
            Zugzwang::new(Field::new(3, 3), true, 1),
        ];

        let mut expected_result: BTreeMap<u8, Vec<Zugzwang>> = BTreeMap::new();
        expected_result.insert(
            2,
            vec![
                Zugzwang::new(Field::new(2, 3), false, 1),
                Zugzwang::new(Field::new(2, 4), false, 2),
            ],
        );
        expected_result.insert(3, vec![Zugzwang::new(Field::new(3, 3), true, 1)]);

        assert_eq!(expected_result, sort_zugzwang_list(zugzwang_list))
    }

    #[test]
    fn evaluate_zugzwang_list_simple_test() {
        let zugzwang_list = vec![
            Zugzwang::create(Field::new(2, 3), 1),
            Zugzwang::create(Field::new(2, 3), 1),
            Zugzwang::create(Field::new(2, 4), 2),
        ];

        assert_eq!(
            -1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, true)
        );
        assert_eq!(1, evaluate_zugzwang_positions(zugzwang_list, 2, false));

        let zugzwang_list = vec![
            Zugzwang::create(Field::new(2, 3), 1),
            Zugzwang::create(Field::new(2, 3), 1),
            Zugzwang::create(Field::new(2, 4), 1),
        ];

        assert_eq!(
            1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, false)
        );
        assert_eq!(-1, evaluate_zugzwang_positions(zugzwang_list, 2, true));
    }

    #[test]
    fn evaluate_horizontal_zugzwang_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 1, 0, 1, 0, 0, 0],
            [2, 2, 0, 2, 0, 0, 0],
            [1, 1, 0, 1, 2, 0, 0],
            [2, 2, 0, 1, 2, 0, 0],
        ];

        let game_board = GameBoard::from(grid);
        assert_eq!(
            -100012000,
            evaluation(&game_board, &mut Default::default(), 2, true, true)
        );
        assert_eq!(
            100012000,
            evaluation(&game_board, &mut Default::default(), 1, false, true)
        );
    }

    #[test]
    fn check_sequence_horizontal_zugzwang_detection_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 1, 0, 1, 0, 0, 0],
            [2, 2, 0, 2, 0, 0, 0],
            [1, 1, 0, 1, 0, 0, 0],
        ];

        let mut x = 0;
        assert_eq!(
            Some(Zugzwang::new(Field::new(2, 4), true, 2)),
            check_sequence_horizontal(&grid, 2, 4, &mut x, 4).1
        );
    }

    #[test]
    fn check_sequence_diagonal_zugzwang_detection_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 2, 0, 0, 0],
            [1, 1, 0, 1, 2, 0, 0],
            [1, 2, 0, 1, 0, 0, 0],
            [2, 1, 0, 1, 0, 0, 2],
        ];

        assert_eq!(
            Some(Zugzwang::new(Field::new(5, 4), true, 2)),
            check_sequence_diagonal(&grid, 2, 4, 3, 2, &mut Vec::new()).1
        );
    }

    #[test]
    fn evaluate_diagonal_zugzwang_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 2, 1, 1, 2, 0],
            [0, 0, 1, 2, 1, 2, 0],
            [0, 0, 1, 2, 1, 1, 0],
            [0, 0, 1, 1, 2, 2, 0],
            [0, 0, 2, 2, 1, 1, 0],
            [0, 0, 1, 1, 2, 2, 0],
        ];

        let mut zugzwang_list: Vec<Zugzwang> = Vec::new();
        let _x = evaluate_threats(&grid, 1, 4, &mut zugzwang_list);
        let expected_zugzwang_list = [
            Zugzwang::new(Field::new(6, 3), false, 1),
            Zugzwang::new(Field::new(1, 1), false, 1),
        ];
        assert_eq!(expected_zugzwang_list, zugzwang_list.as_slice());

        let game_board = GameBoard::from(grid);
        assert_eq!(
            -100026000,
            evaluation(&game_board, &mut Default::default(), 2, false, true)
        );
        assert_eq!(
            100026000,
            evaluation(&game_board, &mut Default::default(), 1, true, true)
        );
    }

    #[test]
    fn check_sequence_diagonal_mirrored_zugzwang_detection_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [1, 0, 2, 0, 2, 0, 0],
            [1, 2, 2, 1, 0, 0, 0],
            [2, 1, 2, 1, 1, 0, 2],
        ];

        assert_eq!(
            Some(Zugzwang::new(Field::new(3, 2), true, 2)),
            check_sequence_diagonal_mirrored(&grid, 2, 4, 3, 2, &mut Vec::new()).1
        );
    }

    #[test]
    fn evaluate_diagonal_mirrored_zugzwang_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 2],
            [0, 0, 0, 0, 1, 0, 2],
            [0, 0, 1, 1, 0, 0, 2],
            [0, 0, 1, 2, 0, 0, 1],
        ];

        let mut zugzwang_list: Vec<Zugzwang> = Vec::new();
        let _x = evaluate_threats(&grid, 1, 4, &mut zugzwang_list);
        let expected_zugzwang_list = [Zugzwang::new(Field::new(5, 2), true, 1)];
        assert_eq!(expected_zugzwang_list, zugzwang_list.as_slice());

        let game_board = GameBoard::from(grid);
        assert_eq!(
            -100016000,
            evaluation(&game_board, &mut Default::default(), 2, true, true)
        );
        assert_eq!(
            100016000,
            evaluation(&game_board, &mut Default::default(), 1, false, true)
        );
    }

    #[test]
    fn evaluate_special_case_test() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 2, 2, 2, 0],
            [0, 0, 0, 1, 1, 1, 0],
            [0, 0, 1, 1, 1, 2, 0],
            [0, 0, 2, 2, 1, 1, 0],
            [0, 0, 2, 1, 2, 1, 0],
            [0, 0, 2, 2, 1, 2, 0],
        ];

        let game_board = GameBoard::from(grid);

        // Liste aller Zugzwänge
        let mut zugzwang_list: Vec<Zugzwang> = Vec::new();

        let _ = evaluate_game_position(&game_board, 2, &mut zugzwang_list);
        let _ = evaluate_game_position(&game_board, 1, &mut zugzwang_list);

        assert_eq!(
            [
                Zugzwang::new(Field::new(2, 0), true, 2),
                Zugzwang::new(Field::new(6, 0), true, 2),
                Zugzwang::new(Field::new(2, 0), true, 1),
                Zugzwang::new(Field::new(6, 1), false, 1),
                Zugzwang::new(Field::new(1, 2), true, 1),
            ],
            zugzwang_list.as_slice()
        );

        assert_eq!(
            100040000,
            evaluation(
                &GameBoard::from(grid),
                &mut Default::default(),
                1,
                true,
                true
            )
        );
    }

    #[test]
    fn evaluate_multiple_zugwzang_test() {
        // Blue (Player 1) to move
        let grid: [[u8; 7]; 6] = [
            [0, 0, 2, 1, 0, 0, 0],
            [1, 0, 1, 1, 2, 0, 0],
            [2, 0, 2, 2, 1, 0, 0],
            [2, 0, 1, 1, 2, 0, 0],
            [2, 0, 2, 2, 1, 0, 1],
            [1, 0, 2, 1, 2, 0, 1],
        ];

        let game_board = GameBoard::from(grid);

        // Liste aller Zugzwänge
        let mut zugzwang_list: Vec<Zugzwang> = Vec::new();

        let _ = evaluate_game_position(&game_board, 2, &mut zugzwang_list);
        let _ = evaluate_game_position(&game_board, 1, &mut zugzwang_list);
        assert_eq!(
            [
                Zugzwang::new(Field::new(1, 2), true, 2),
                Zugzwang::new(Field::new(1, 4), true, 2),
                Zugzwang::new(Field::new(5, 2), true, 2),
                Zugzwang::new(Field::new(1, 1), false, 1),
                Zugzwang::new(Field::new(5, 3), false, 1)
            ],
            zugzwang_list.as_slice()
        );

        assert_eq!(
            -1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, false)
        );
        assert_eq!(1, evaluate_zugzwang_positions(zugzwang_list, 1, true));
    }

    /*
       ------------ ZUGZWANG POSITION EVALUATION TESTS ------------
    */

    #[test]
    fn simulate_zugzwang_positions_simple_row_test() {
        let zugzwang_list = vec![
            Zugzwang::create(Field::new(0, 3), 2),
            Zugzwang::create(Field::new(0, 2), 1),
        ];

        assert_eq!(
            1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, true)
        );
        assert_eq!(
            0,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, false)
        );
        assert_eq!(
            -1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, false)
        );
        assert_eq!(
            0,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, true)
        );
    }

    #[test]
    fn simulate_zugzwang_positions_two_rows_test1() {
        let zugzwang_list = vec![
            Zugzwang::create(Field::new(0, 0), 2),
            Zugzwang::create(Field::new(0, 3), 2),
            Zugzwang::create(Field::new(1, 0), 1),
            Zugzwang::create(Field::new(1, 3), 1),
        ];

        assert_eq!(
            -1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, true)
        );
        assert_eq!(
            1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, false)
        );
    }

    #[test]
    fn simulate_zugzwang_positions_two_rows_test2() {
        let zugzwang_list = vec![
            Zugzwang::create(Field::new(0, 3), 1),
            Zugzwang::create(Field::new(1, 1), 2),
            Zugzwang::create(Field::new(1, 4), 2),
        ];

        assert_eq!(
            0,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, true)
        );
        assert_eq!(
            0,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, false)
        );
    }

    #[test]
    fn simulate_zugzwang_positions_two_rows_test3() {
        let zugzwang_list = vec![
            Zugzwang::create(Field::new(0, 0), 1),
            Zugzwang::create(Field::new(0, 3), 1),
            Zugzwang::create(Field::new(1, 2), 2),
            Zugzwang::create(Field::new(1, 4), 2),
        ];

        assert_eq!(
            1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, true)
        );
        assert_eq!(
            -1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, false)
        );
    }

    #[test]
    fn simulate_zugzwang_positions_three_rows_test() {
        let zugzwang_list = vec![
            Zugzwang::create(Field::new(0, 3), 1),
            Zugzwang::create(Field::new(1, 4), 2),
            Zugzwang::create(Field::new(2, 3), 2),
        ];

        assert_eq!(
            -1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, true)
        );
        assert_eq!(
            1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, false)
        );
    }

    #[test]
    fn simulate_zugzwang_positions_four_rows_test() {
        let zugzwang_list = vec![
            Zugzwang::create(Field::new(0, 4), 2),
            Zugzwang::create(Field::new(1, 3), 2),
            Zugzwang::create(Field::new(2, 2), 1),
            Zugzwang::create(Field::new(3, 1), 1),
        ];

        assert_eq!(
            -1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, true)
        );
        assert_eq!(
            1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, false)
        );
    }

    #[test]
    fn evaluate_horizontal_zugwzang_test() {
        // Blue (Player 1) to move
        let grid: [[u8; 7]; 6] = [
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
            [0, 0, 2, 2, 2, 0, 0],
            [1, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0],
        ];

        let game_board = GameBoard::from(grid);

        // Liste aller Zugzwänge
        let mut zugzwang_list: Vec<Zugzwang> = Vec::new();

        let _ = evaluate_game_position(&game_board, 2, &mut zugzwang_list);
        let _ = evaluate_game_position(&game_board, 1, &mut zugzwang_list);

        assert_eq!(
            1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, true)
        );
        assert_eq!(-1, evaluate_zugzwang_positions(zugzwang_list, 1, false));
    }

    #[test]
    fn evaluate_zugzwang_positions_complex_example_test() {
        let zugzwang_list = vec![
            Zugzwang::create(Field::new(1, 1), 1),
            Zugzwang::create(Field::new(1, 2), 2),
            Zugzwang::create(Field::new(1, 3), 1),
            Zugzwang::create(Field::new(5, 0), 2),
            Zugzwang::create(Field::new(5, 1), 1),
            Zugzwang::create(Field::new(5, 2), 2),
            Zugzwang::create(Field::new(5, 3), 1),
        ];

        assert_eq!(
            1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, true)
        );
        assert_eq!(
            -1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, false)
        );
    }

    #[test]
    fn evaluate_zugzwang_positions_shared_zugzwang_test() {
        let zugzwang_list = vec![
            Zugzwang::create(Field::new(0, 4), 2),
            Zugzwang::create(Field::new(0, 3), 1),
            Zugzwang::create(Field::new(0, 2), 2),
            Zugzwang::create(Field::new(0, 2), 1),
            Zugzwang::create(Field::new(0, 1), 2),
            Zugzwang::create(Field::new(0, 0), 1),
        ];

        assert_eq!(
            -1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, true)
        );
        assert_eq!(
            1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, false)
        );
    }

    #[test]
    fn evaluate_zugzwang_positions_example2_test() {
        let zugzwang_list = vec![
            Zugzwang::create(Field::new(1, 1), 2),
            Zugzwang::create(Field::new(5, 1), 2),
            Zugzwang::create(Field::new(1, 2), 1),
            Zugzwang::create(Field::new(5, 2), 1),
            Zugzwang::create(Field::new(0, 4), 1),
        ];

        assert_eq!(
            -1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, true)
        );
        assert_eq!(
            1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, false)
        );
    }

    #[test]
    fn evaluate_zugzwang_positions_example3_test() {
        let expected_zugzwang_list = vec![
            Zugzwang::create(Field::new(1, 2), 1),
            Zugzwang::create(Field::new(6, 3), 2),
        ];

        // Blue (Player 1) to move
        let grid: [[u8; 7]; 6] = [
            [0, 0, 1, 2, 1, 2, 0],
            [0, 0, 1, 2, 2, 1, 0],
            [0, 0, 1, 1, 1, 2, 0],
            [0, 0, 2, 2, 2, 1, 0],
            [0, 2, 2, 1, 1, 2, 0],
            [0, 1, 1, 2, 2, 2, 1],
        ];

        let game_board = GameBoard::from(grid);

        // Liste aller Zugzwänge
        let mut zugzwang_list: Vec<Zugzwang> = Vec::new();

        let _ = evaluate_game_position(&game_board, 1, &mut zugzwang_list);
        let _ = evaluate_game_position(&game_board, 2, &mut zugzwang_list);

        assert_eq!(expected_zugzwang_list, zugzwang_list.as_slice());

        assert_eq!(
            1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 2, true)
        );
        assert_eq!(
            -1,
            evaluate_zugzwang_positions(zugzwang_list.clone(), 1, false)
        );

        assert_eq!(
            -100014000,
            evaluation(&game_board, &mut Default::default(), 1, false, true)
        );
    }

    #[test]
    fn test_difficulty_setting() {
        let grid: [[u8; 7]; 6] = [
            [0, 0, 1, 2, 1, 2, 0],
            [0, 0, 1, 2, 2, 1, 0],
            [0, 0, 1, 1, 1, 2, 0],
            [0, 0, 2, 2, 2, 1, 0],
            [0, 2, 2, 1, 1, 2, 0],
            [0, 1, 1, 2, 2, 2, 1],
        ];

        let mut game_board = GameBoard::from(grid);
        assert_eq!(
            -100014000,
            evaluation(&game_board, &mut Default::default(), 1, false, true)
        );

        assert_eq!(
            -14000,
            evaluation(&game_board, &mut Default::default(), 1, false, false)
        );

        assert_ne!(
            next_move(&mut game_board, true, &Difficulty::from_int(0)),
            next_move(&mut game_board, true, &Difficulty::from_int(3))
        )
    }
}
