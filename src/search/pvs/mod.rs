use primitive_enum::primitive_enum;

use crate::evaluation::ValueScore;

pub mod score_table;
pub mod window;

const MATE_SCORE: ValueScore = ValueScore::MIN + 1;

primitive_enum! { NodeType u8;
    PVNode,
    AllNode,
    CutNode,
}
