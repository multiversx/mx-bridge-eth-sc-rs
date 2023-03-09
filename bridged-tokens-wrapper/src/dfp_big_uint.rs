multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub struct DFPBigUint<M: ManagedTypeApi> {
    bu: BigUint<M>,
    num_decimals: u32,
}

impl<M: ManagedTypeApi> DFPBigUint<M> {
    pub fn from_raw(bu: BigUint<M>, num_decimals: u32) -> Self {
        DFPBigUint { bu, num_decimals }
    }

    pub fn convert(&self, decimals: u32) -> BigUint<M> {
        if self.num_decimals < decimals {
            let diff_decimals = decimals - self.num_decimals;
            return self.bu.clone() * 10u32.pow(diff_decimals);
        }
        if self.num_decimals > decimals {
            let diff_decimals = self.num_decimals - decimals;
            return self.bu.clone() / 10u32.pow(diff_decimals);
        }
        self.bu.clone()
    }
}
