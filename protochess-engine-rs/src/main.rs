//#[macro_use] extern crate scan_rules;

use std::io::Write;

use protochess_engine_rs::{Engine, MoveInfo, MakeMoveResult};

pub fn main() {
    
    // Some interesting FENs:
    // "R3b3/4k3/2n5/p4p1p/4p3/2B5/1PP2PPP/5K2 w - - 10 36"
    // "rnbqkbnr/nnnnnnnn/rrrrrrrr/8/8/8/QQQQQQQQ/RNBQKBNR w KQkq - 0 1"
    // "rnbqkbnr/pp4pp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    // "r1b3nr/ppqk1Bbp/2pp4/4P1B1/3n4/3P4/PPP2QPP/R4RK1 w - - 1 0"
    // "1Q6/5pk1/2p3p1/1pbbN2p/4n2P/8/r5P1/5K2 b - - 0 1"
    // "rnbqkbnr/pppppppp/8/8/8/8/8/RNBQKBNR w KQkq - 0 1"
    
    
    // Usage: cargo run -- <depth> <fen> <num_ply>
    // By default, <depth> is 12, <fen> is the starting position, and <num_ply> is 500
    // Example: cargo run -- 4 "1Q6/5pk1/2p3p1/1pbbN2p/4n2P/8/r5P1/5K2 b - - 0 1"
    
    let mut pgn_file = std::fs::File::create("pgn.txt").expect("create failed");

    
    let args: Vec<String> = std::env::args().collect();
    let mut fixed_depth = true;
    let mut depth = 12;
    let mut max_ply = 500;
    if args.len() > 3 {
        max_ply = args[3].parse::<u32>().unwrap();
    }
    let mut engine = {
        if args.len() > 2 && args[2] != "default" {
            let fen = &args[2];
            print_pgn_header(fen, &mut pgn_file);
            Engine::from_fen(fen).unwrap()
        } else {
            Engine::default()
        }
    };
    if args.len() > 1 {
        if args[1].contains('t') {
            fixed_depth = false;
            depth = args[1].replace('t', "").parse::<u8>().unwrap();
        } else {
            depth = args[1].parse::<u8>().unwrap();
        }
    }
    
    println!("Start Position:\n{engine}");
    println!("\n----------------------------------------\n");
    
    let start = instant::Instant::now();
    for ply in 0..max_ply {
        let mv = {
            if fixed_depth {
                engine.get_best_move(depth).unwrap().0
            } else {
                engine.get_best_move_timeout(depth as u64).unwrap().0
            }
        };
        println!("\n========================================\n");
        println!("(Time since start: {:?})", start.elapsed());
        println!("PLY: {ply} Engine plays:\n");
        print_pgn(&mut pgn_file, ply, &to_long_algebraic_notation(&mv, &engine));
        let result = engine.make_move(&mv);
        println!("{engine}\n");
        match result {
            MakeMoveResult::Ok => {
                println!("----------------------------------------\n");
            },
            MakeMoveResult::IllegalMove => {
                panic!("An illegal move was made");
            },
            MakeMoveResult::Checkmate{winner} => {
                println!("CHECKMATE! {} wins!", if winner == 0 { "White" } else { "Black" });
                break;
            },
            MakeMoveResult::LeaderCaptured{winner} => {
                println!("KING HAS BEEN CAPTURED! {} wins!", if winner == 0 { "White" } else { "Black" });
                break;
            },
            MakeMoveResult::PieceInWinSquare{winner} => {
                println!("KING IN WINNING SQUARE! {} wins!", if winner == 0 { "White" } else { "Black" });
                break;
            },
            MakeMoveResult::CheckLimit{winner} => {
                println!("CHECK LIMIT REACHED! {} wins!", if winner == 0 { "White" } else { "Black" });
                break;
            },
            MakeMoveResult::Stalemate{winner: Some(winner)} => {
                println!("STALEMATE! {} wins!", if winner == 0 { "White" } else { "Black" });
                break;
            },
            MakeMoveResult::Stalemate{winner: None} => {
                println!("DRAW BY STALEMATE!");
                break;
            },
            MakeMoveResult::Repetition => {
                println!("DRAW BY REPETITION!");
                break;
            },
        }
    }
}

fn print_pgn_header(fen: &str, pgn_file: &mut std::fs::File) {
    const STD_FEN_SIZE: usize = 6;
    let fen_vec: Vec<_> = fen.split_whitespace().collect();
    assert!(fen_vec.len() >= STD_FEN_SIZE);
    if fen_vec.len() > STD_FEN_SIZE {
        let variant = fen_vec.last().unwrap();
        pgn_file.write_all(format!("[Variant \"{}\"]\n", variant).as_bytes()).unwrap();
    }
    let std_fen = fen_vec[0..STD_FEN_SIZE].join(" ");
    pgn_file.write_all(format!("[FEN \"{}\"]\n\n", std_fen).as_bytes()).unwrap();
}

fn print_pgn(pgn_file: &mut std::fs::File, ply: u32, move_str: &str) {
    if (ply % 2) == 0 {
        let round = format!("{}. ", ply/2 + 1);
        pgn_file.write_all(round.as_bytes()).expect("write failed");
    }
    pgn_file.write_all(move_str.as_bytes()).expect("write failed");
    pgn_file.write_all(b" ").expect("write failed");
}

pub fn to_long_algebraic_notation(mv: &MoveInfo, engine: &Engine) -> String {
    // Long algebraic notation for mv
    let move_string = format!("{}{}{}{}", (b'a' + mv.from.0) as char, mv.from.1 + 1, (b'a' + mv.to.0) as char, mv.to.1 + 1);
    
    if let Some(prom) = mv.promotion {
        let prom_char = engine.get_piece_char(prom).unwrap().to_ascii_uppercase();
        return format!("{move_string}={prom_char}");
    }
    
    let piece_id = engine.get_piece_at(mv.from).unwrap();
    let piece_char = engine.get_piece_char(piece_id).unwrap().to_ascii_uppercase();
    let result = {
        if piece_char == 'P' {
            move_string
        } else {
            // If the piece is not a pawn, write the piece letter
            format!("{piece_char}{move_string}")
        }
    };
    
    match result.as_str() {
        "Ke1h1" | "Ke8h8" => "O-O".to_string(),
        "Ke1a1" | "Ke8a8" => "O-O-O".to_string(),
        _ => result
    }
}
