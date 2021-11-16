elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi, Clone, Copy, PartialEq)]
pub enum UserRole {
    None,
    _Proposer,
    BoardMember,
}

impl UserRole {
    #[inline(always)]
    pub fn is_board_member(&self) -> bool {
        matches!(*self, UserRole::BoardMember)
    }
}
