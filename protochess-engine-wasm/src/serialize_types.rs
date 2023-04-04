use protochess_engine_rs::*;
use serde_wasm_bindgen::{to_value, from_value};
use wasm_bindgen::prelude::*;
use super::utils::SerVec;


macro_rules! generate_wrapper {
    ($wrapper_name:ident, $wrapped_type:ident, [$($field:ident, $type:ty),*]) => {
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        #[wasm_bindgen(inspectable)]
        #[must_use]
        pub struct $wrapper_name {
            $( $field: $type ),*
        }
        impl $wrapper_name {
            pub fn to_js(val: $wrapped_type) -> JsValue {
                to_value(&Self::from(val)).unwrap()
            }
            pub fn from_js(val: JsValue) -> Result<$wrapped_type, String> {         
                const TYPE_NAME: &str = stringify!($wrapper_name);       
                let wrapper = from_value::<$wrapper_name>(val)
                    .map_err(|e| format!("Argument must be of type {TYPE_NAME}. {e}"))?;
                Ok(wrapper.into())
            }
        }
        impl From<$wrapped_type> for $wrapper_name {
            fn from(val: $wrapped_type) -> Self {
                $wrapper_name {
                    $( $field: (val.$field).into() ),*
                }
            }
        }
        impl From<$wrapper_name> for $wrapped_type {
            fn from(val: $wrapper_name) -> Self {
                $wrapped_type {
                    $( $field: (val.$field).into() ),*
                }
            }
        }
    }
}

generate_wrapper!(MoveInfoSer, MoveInfo, [
    from, (u8, u8),
    to, (u8, u8),
    promotion, Option<char>
]);

generate_wrapper!(MoveListSer, MoveList, [
    x, u8,
    y, u8,
    moves, SerVec<MoveInfoSer>
]);


generate_wrapper!(MakeMoveResultSer, MakeMoveResult, [
    flag, String,
    winner, String,
    exploded, SerVec<(u8, u8)>
]);


#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveInfoWithEvalSer {
    move_info: MoveInfoSer,
    evaluation: i32,
}
impl MoveInfoWithEvalSer {
    pub fn to_js(mv: MoveInfo, evaluation: i32) -> JsValue {
        let val = MoveInfoWithEvalSer {
            move_info: MoveInfoSer::from(mv),
            evaluation
        };
        to_value(&val).unwrap()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveInfoWithEvalDepthSer {
    move_info: MoveInfoSer,
    evaluation: i32,
    depth: u8,
}
impl MoveInfoWithEvalDepthSer {
    pub fn to_js(mv: MoveInfo, evaluation: i32, depth: u8) -> JsValue {
        let val = MoveInfoWithEvalDepthSer {
            move_info: MoveInfoSer::from(mv),
            evaluation,
            depth
        };
        to_value(&val).unwrap()
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MakeMoveResultWithDepthSer {
    make_move_result: MakeMoveResultSer,
    depth: u8
}
impl MakeMoveResultWithDepthSer {
    pub fn to_js(mmr: MakeMoveResult, depth: u8) -> JsValue {
        let val = MakeMoveResultWithDepthSer {
            make_move_result: mmr.into(),
            depth
        };
        to_value(&val).unwrap()
    }
}

generate_wrapper!(PieceDefinitionSer, PieceDefinition, [
    ids, [Option<char>; 2],
    is_leader, bool,
    castle_files, Option<(u8, u8)>,
    is_castle_rook, bool,
    explodes, bool,
    explosion_deltas, Vec<(i8, i8)>,
    immune_to_explosion, bool,
    promotion_squares, Vec<(u8, u8)>,
    promo_vals, [Vec<char>; 2],
    double_jump_squares, Vec<(u8, u8)>,
    attack_sliding_deltas, Vec<Vec<(i8, i8)>>,
    attack_jump_deltas, Vec<(i8, i8)>,
    attack_north, bool,
    attack_south, bool,
    attack_east, bool,
    attack_west, bool,
    attack_northeast, bool,
    attack_northwest, bool,
    attack_southeast, bool,
    attack_southwest, bool,
    translate_jump_deltas, Vec<(i8, i8)>,
    translate_sliding_deltas, Vec<Vec<(i8, i8)>>,
    translate_north, bool,
    translate_south, bool,
    translate_east, bool,
    translate_west, bool,
    translate_northeast, bool,
    translate_northwest, bool,
    translate_southeast, bool,
    translate_southwest, bool,
    win_squares, Vec<(u8, u8)>,
    display_name, String,
    image_urls, [Option<String>; 2]
]);


generate_wrapper!(PiecePlacementSer, PiecePlacement, [
    piece_id, char,
    x, u8,
    y, u8,
    // True if it has not moved. This is an option so that JS can leave it as undefined
    can_castle, Option<bool>
]);

generate_wrapper!(GlobalRulesSer, GlobalRules, [
    capturing_is_forced, bool,
    check_is_forbidden, bool,
    stalemated_player_loses, bool,
    invert_win_conditions, bool,
    repetitions_draw, u8,
    checks_to_lose, u8
]);

generate_wrapper!(GameStateSer, InitialState, [
    piece_types, SerVec<PieceDefinitionSer>,
    board_width, u8,
    board_height, u8,
    invalid_squares, Vec<(u8, u8)>,
    pieces, SerVec<PiecePlacementSer>,
    player_to_move, u8,
    ep_square_and_victim, Option<((u8, u8), (u8, u8))>,
    times_in_check, Option<[u8; 2]>,
    global_rules, GlobalRulesSer,
    variant_display_name, Option<String>
]);

generate_wrapper!(GameStateGuiSer, GameStateGui, [
    state, GameStateSer,
    fen, String,
    in_check, bool
]);
