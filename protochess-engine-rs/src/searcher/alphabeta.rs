use instant::Instant;

use crate::{Position, MoveGen};
use crate::types::{Move, Depth, Centipawns};

use super::Searcher;
use super::eval;
use super::transposition_table::{Entry, EntryFlag};

pub const GAME_OVER_SCORE: Centipawns = -1000000;

impl Searcher {
    pub fn search(&mut self, pos: &mut Position, depth: Depth, end_time: &Instant) -> Result<Centipawns,()> {
        // Use -MAX instead of MIN to avoid overflow when negating
        self.alphabeta(pos, depth, -Centipawns::MAX, Centipawns::MAX, true, end_time)
    }
    
    // alpha is the best score that I can currently guarantee at this level or above.
    // beta is the worst score for me that the opponent can currently guarantee at this level or above.
    fn alphabeta(&mut self,
            pos: &mut Position,
            mut depth: Depth,
            mut alpha: Centipawns,
            beta: Centipawns,
            do_null: bool,
            end_time: &Instant
        ) -> Result<Centipawns,()>
    {
        // If there is repetition, the result is always a draw
        if pos.num_repetitions() >= 3 {
            return Ok(0);
        }
        
        if depth == 0 {
            return self.quiesce(pos, alpha, beta, end_time, 0);
        }
        
        if pos.leader_is_captured() {
            return Ok(self.checkmate_score(depth as i16))
        }
        
        self.nodes_searched += 1;
        // Check for timeout periodically
        if self.nodes_searched.trailing_zeros() >= 20 && Instant::now() >= *end_time {
            return Err(());
        }

        let is_pv = alpha != beta - 1;
        if let Some(entry) = self.transposition_table.retrieve(pos.get_zobrist()) {
            if entry.depth >= depth {
                match entry.flag {
                    EntryFlag::Exact => {
                        if entry.value < alpha {
                            self.principal_variation[self.current_depth(depth as i16) as usize] = entry.mv;
                            return Ok(alpha);
                        }
                        if entry.value >= beta {
                            return Ok(beta);
                        }
                        return Ok(entry.value);
                    }
                    EntryFlag::Beta => {
                        if !is_pv && beta <= entry.value {
                            return Ok(beta);
                        }
                    }
                    EntryFlag::Alpha => {
                        if !is_pv && alpha >= entry.value {
                            self.principal_variation[self.current_depth(depth as i16) as usize] = entry.mv;
                            return Ok(alpha);
                        }
                    }
                    EntryFlag::Null => {}
                }
            }
        }

        //Null move pruning
        if !is_pv && do_null && depth > 3 && eval::can_do_null_move(pos) && !MoveGen::in_check(pos) {
            pos.make_move(Move::null());
            let nscore = -self.alphabeta(pos, depth - 3, -beta, -beta + 1, false, end_time)?;
            pos.unmake_move();
            if nscore >= beta {
                return Ok(beta);
            }
        }
        
        // TODO: Remove best_move and best_score
        let mut best_move = Move::null();
        let mut num_legal_moves = 0;
        let old_alpha = alpha;
        let mut best_score = -Centipawns::MAX; // Use -MAX instead of MIN to avoid overflow when negating
        let in_check = MoveGen::in_check(pos);
        if in_check {
            // If in check, extend search by 1 ply
            depth += 1;
        }
        
        // Get potential moves, sorted by move ordering heuristics (try the most promising moves first)
        let moves = MoveGen::get_pseudo_moves(pos);
        for (_move_score, mv) in self.sort_moves_by_score(pos, moves, depth) {
            
            if !MoveGen::make_move_only_if_legal(&mv, pos) {
                continue;
            }

            num_legal_moves += 1;
            let mut score: Centipawns;
            if num_legal_moves == 1 {
                score = -self.alphabeta(pos, depth - 1, -beta, -alpha, true, end_time)?;
            } else {
                //Try late move reduction
                if num_legal_moves > 4 && mv.is_quiet() && !is_pv && depth >= 5 && !in_check {
                    //Null window search with depth - 2
                    let mut reduced_depth = depth - 2;
                    if num_legal_moves > 10 {
                        reduced_depth = depth - 3;
                    }
                    score = -self.alphabeta(pos, reduced_depth, -alpha - 1, -alpha, true, end_time)?;
                } else {
                    //Cannot reduce, proceed with standard PVS
                    score = alpha + 1;
                }

                if score > alpha {
                    //PVS
                    //Null window search
                    score = -self.alphabeta(pos, depth - 1, -alpha - 1, -alpha, true, end_time)?;
                    //Re-search if necessary
                    if score > alpha && score < beta {
                        score = -self.alphabeta(pos, depth - 1, -beta, -score, true, end_time)?;
                    }
                }

            }

            pos.unmake_move();

            if score > best_score {
                best_score = score;
                best_move = mv;

                if score > alpha {
                    if score >= beta {
                        // Record new killer moves
                        self.update_killers(depth, mv);
                        // Beta cutoff, store in transpositon table
                        self.transposition_table.insert(pos.get_zobrist(), Entry{
                            key: pos.get_zobrist(),
                            flag: EntryFlag::Beta,
                            value: beta,
                            mv,
                            depth,
                        });
                        return Ok(beta);
                    }
                    alpha = score;

                    // History heuristic
                    self.update_history_heuristic(depth, &mv);
                }
            }
        }

        if num_legal_moves == 0 {
            return if in_check {
                // No legal moves and in check: Checkmate
                Ok(self.checkmate_score(depth as i16))
            } else {
                // No legal moves but also not in check: Stalemate
                Ok(0)
            };
        }

        if alpha != old_alpha {
            //Alpha improvement, record PV
            self.transposition_table.insert(pos.get_zobrist(), Entry{
                key: pos.get_zobrist(),
                flag: EntryFlag::Exact,
                value: best_score,
                mv: best_move,
                depth,
            });
        } else {
            self.transposition_table.insert(pos.get_zobrist(), Entry{
                key: pos.get_zobrist(),
                flag: EntryFlag::Alpha,
                value: alpha,
                mv: best_move,
                depth,
            });
        }
        
        self.principal_variation[self.current_depth(depth as i16) as usize] = best_move;
        
        Ok(alpha)
    }


    // Keep seaching, but only consider capture moves (avoid horizon effect)
    // depth starts at 0 and is decreased by 1 for each ply
    fn quiesce(&mut self, pos: &mut Position, mut alpha: Centipawns, beta: Centipawns, end_time: &Instant, depth: i16) -> Result<Centipawns,()> {
        
        if pos.leader_is_captured() {
            return Ok(self.checkmate_score(depth));
        }
        
        self.nodes_searched += 1;
        // Check for timeout periodically
        if self.nodes_searched.trailing_zeros() >= 20 && Instant::now() >= *end_time {
            return Err(());
        }
        
        let score = eval::evaluate(pos);
        
        if score >= beta {
            return Ok(beta);
        }
        if score > alpha {
            alpha = score;
        }

        // Get only captures, sorted by move ordering heuristics (try the most promising moves first)
        let moves = MoveGen::get_capture_moves(pos);
        for (_move_score, mv) in self.sort_moves_by_score(pos, moves, 0) {
            // This is a capture move, so there is no need to check for repetition
            if !MoveGen::make_move_only_if_legal(&mv, pos) {
                continue;
            }
            let score = -self.quiesce(pos, -beta, -alpha, end_time, depth - 1)?;
            pos.unmake_move();

            if score >= beta {
                return Ok(beta);
            }
            if score > alpha {
                alpha = score;
            }
        }
        Ok(alpha)
    }

    #[inline]
    fn update_killers(&mut self, depth: Depth, mv: Move) {
        if !mv.is_capture() && mv != self.killer_moves[depth as usize][0] && mv != self.killer_moves[depth as usize][1] {
            self.killer_moves[depth as usize][1] = self.killer_moves[depth as usize][0];
            self.killer_moves[depth as usize][0] = mv;
        }
    }

    #[inline]
    fn update_history_heuristic(&mut self, depth: Depth, mv:&Move) {
        if !mv.is_capture() {
            self.history_moves
                [mv.get_from() as usize]
                [mv.get_to() as usize] += depth as Centipawns;
        }
    }

    #[inline]
    fn sort_moves_by_score<I: Iterator<Item=Move>>(&mut self, pos: &mut Position, moves: I, depth: Depth) -> Vec<(Centipawns, Move)> {
        // Limit depth to the size of the killer moves array
        let depth = std::cmp::min(self.killer_moves.len() - 1, depth as usize);
        let killer_moves_at_depth = &self.killer_moves[depth];
        let mut moves_and_score: Vec<(Centipawns, Move)> = moves.map(|mv| 
            (eval::score_move(&self.history_moves, killer_moves_at_depth, pos, &mv), mv))
            .collect();

        // Assign PV/hash moves to Centipawns::MAX (search first in the PV)
        if let Some(entry) = self.transposition_table.retrieve(pos.get_zobrist()) {
            let best_move = &entry.mv;
            for (score, mv) in &mut moves_and_score {
                if mv == best_move {
                    *score = Centipawns::MAX;
                }
            }
        }
        
        // Sort moves by decreasing score
        moves_and_score.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        
        moves_and_score
    }
    
    #[inline]
    fn checkmate_score(&self, depth: i16) -> Centipawns {
        // A checkmate is effectively -inf, but if we are losing we prefer the longest sequence
        // Add 1 centipawn per ply to the score to prefer shorter checkmates (or longer when losing)
        GAME_OVER_SCORE + self.current_depth(depth) as Centipawns
    }
    
    #[inline]
    fn current_depth(&self, depth: i16) -> i16 {
        self.current_searching_depth as i16 - depth
    }
    

}