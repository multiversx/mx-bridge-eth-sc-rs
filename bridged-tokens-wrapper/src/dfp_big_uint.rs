elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(Clone)]
pub struct DFPBigUint<M: ManagedTypeApi> {
    bu: BigUint<M>,
    num_decimals: u32,
}

impl<M: ManagedTypeApi> DFPBigUint<M> {
    pub fn from_raw(bu: BigUint<M>, num_decimals: u32) -> Self {
        DFPBigUint { bu, num_decimals }
    }

    pub fn convert(&self, decimals: u32) -> Self {
        if self.num_decimals < decimals {
            let diff_decimals = decimals - self.num_decimals;
            return DFPBigUint {
                bu: self.bu.clone() * 10u32.pow(diff_decimals),
                num_decimals: decimals,
            };
        }
        if self.num_decimals > decimals {
            let diff_decimals = self.num_decimals - decimals;
            return DFPBigUint {
                bu: self.bu.clone() / 10u32.pow(diff_decimals),
                num_decimals: decimals,
            };
        }
        self.clone()
    }

    pub fn to_raw(&self) -> BigUint<M> {
        self.bu.clone()
    }
}
