use primitive_enum::primitive_enum;

pub mod score_table;
pub mod window;

primitive_enum! { NodeType u8;
    PVNode,
    AllNode,
    CutNode,
}
