use multiversx_sc::derive_imports::*;

#[type_abi]
#[derive(TopEncode, TopDecode, Clone, Copy, PartialEq)]
pub enum UserRole {
    None,
    BoardMember,
}

impl UserRole {
    #[inline(always)]
    pub fn is_board_member(&self) -> bool {
        matches!(*self, UserRole::BoardMember)
    }
}
