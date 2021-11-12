elrond_wasm::derive_imports!();

#[derive(TopEncode, TopDecode, TypeAbi, Clone, Copy, PartialEq)]
pub enum UserRole {
    None,
    _Proposer,
    BoardMember,
}

impl UserRole {
    pub fn can_propose(&self) -> bool {
        matches!(*self, UserRole::BoardMember)
    }

    pub fn can_perform_action(&self) -> bool {
        self.can_propose()
    }

    pub fn can_sign(&self) -> bool {
        self.can_propose()
    }
}
