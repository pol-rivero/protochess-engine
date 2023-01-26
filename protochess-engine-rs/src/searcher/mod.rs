use std::collections::BTreeSet;

use instant::{Instant, Duration};

use crate::types::{Move, Depth, Centipawns, SearchTimeout};
use crate::{Position, MoveGen, err_assert, wrap_res};

mod alphabeta;
mod transposition_table;
pub mod eval;

use transposition_table::TranspositionTable;

#[derive(Debug, Clone)]
pub struct Searcher {
    // The position we are currently searching
    pos: Position,
    //We store two killer moves per ply,
    //indexed by killer_moves[depth][0] or killer_moves[depth][0]
    killer_moves: [[Move;2];64],
    //Indexed by history_moves[side2move][from][to]
    history_moves: [[Centipawns;256];256],
    transposition_table: TranspositionTable,
    // Stats
    nodes_searched: u64,
    max_searching_depth: Depth,
    end_time: Instant,
    principal_variation: [Move; Depth::MAX as usize + 1],
    known_checks: BTreeSet<u64>,
}

impl Searcher {
    fn new(position: &Position) -> Searcher {
        Searcher{
            pos: position.clone(),
            killer_moves: [[Move::null(); 2];64],
            history_moves: [[0;256];256],
            transposition_table: TranspositionTable::new(),
            nodes_searched: 0,
            max_searching_depth: 0,
            end_time: Instant::now(),
            principal_variation: [Move::null(); Depth::MAX as usize + 1],
            known_checks: BTreeSet::new(),
        }
    }
    
    pub fn get_best_move(position: &Position, depth: Depth) -> wrap_res!(Vec<Move>, Centipawns, Depth) {
        // Create a new copy of the heuristics for each search
        // Cannot use u64::MAX due to overflow, 1_000_000 seconds is 11.5 days
        Searcher::new(position).get_best_move_impl(depth, 1_000_000)
    }

    pub fn get_best_move_timeout(position: &Position, time_sec: u64) -> wrap_res!(Vec<Move>, Centipawns, Depth) {
        // Create a new copy of the heuristics for each search
        Searcher::new(position).get_best_move_impl(Depth::MAX, time_sec)
    }
    
    // Run for some time, then return the PV, the position score, and the depth
    fn get_best_move_impl(&mut self, max_depth: Depth, time_sec: u64) -> wrap_res!(Vec<Move>, Centipawns, Depth) {
        err_assert!(!self.pos.leader_is_captured(), "Attempting to get best move but leader is captured");
        err_assert!(MoveGen::count_legal_moves(&mut self.pos) != 0, "Attempting to get best move but there are no legal moves");
        
        // Limit the max depth to 127 to avoid overflow when doubling
        let max_depth = std::cmp::min(max_depth, 127);
        let mut pv = Vec::with_capacity(max_depth as usize);
        let mut pv_depth: Depth = 0;
        let mut pv_score: Centipawns = 0;
        self.known_checks.clear();
        self.end_time = Instant::now() + Duration::from_secs(time_sec);
        
        // Iterative deepening
        for search_depth in 1..=max_depth {
            self.nodes_searched = 0;
            self.max_searching_depth = 2 * search_depth;
            match self.search(search_depth) {
                Ok(score) => {
                    pv.clear();
                    // Copy the pv into a vector
                    for mv in self.principal_variation {
                        if mv.is_null() {
                            break;
                        }
                        pv.push(mv);
                    }
                    // Clean up the pv
                    for i in 0..self.max_searching_depth {
                        self.principal_variation[i as usize] = Move::null();
                    }
                    pv_depth = search_depth;
                    pv_score = score;
                    // Print PV info
                    let diff = -(score.abs() + alphabeta::GAME_OVER_SCORE);
                    let score_str = {
                        if diff < 200 {
                            let sign = if score > 0 { "" } else { "-" };
                            format!("MATE {}{}", sign, (diff+1) / 2)
                        } else {
                            format!("cp {:<4}", score)
                        }
                    };
                    println!("Depth {:<2} Score: {} [nodes: {}]", search_depth, score_str, self.nodes_searched);
                    print!("  PV: ");
                    for m in &pv {
                        print!("{} ", m);
                    }
                    println!();
                },
                Err(SearchTimeout) => {
                    // Thread timed out, return the best move found so far
                    break;
                },
            }

            if Instant::now() >= self.end_time {
                // Return the best move found so far
                break;
            }
        }
        Ok((pv, pv_score, pv_depth))
    }
}
