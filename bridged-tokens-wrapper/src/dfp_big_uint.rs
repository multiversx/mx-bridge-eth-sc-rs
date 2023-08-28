multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(Clone, PartialEq, Eq)]
pub struct DFPBigUint<M: ManagedTypeApi> {
    bu: BigUint<M>,
    num_decimals: u32,
}

impl<M: ManagedTypeApi> DFPBigUint<M> {
    pub fn from_raw(bu: BigUint<M>, num_decimals: u32) -> Self {
        DFPBigUint { bu, num_decimals }
    }

    pub fn convert(self, decimals: u32) -> Self {
        if self.num_decimals == decimals {
            return self;
        }

        let new_bu = if self.num_decimals > decimals {
            let diff_decimals = self.num_decimals - decimals;
            self.bu / 10u64.pow(diff_decimals)
        } else {
            let diff_decimals = decimals - self.num_decimals;
            self.bu * 10u64.pow(diff_decimals)
        };

        DFPBigUint {
            bu: new_bu,
            num_decimals: decimals,
        }
    }

    pub fn trunc(&self) -> Self {
        DFPBigUint {
            bu: self.bu.clone() / 10u64.pow(self.num_decimals),
            num_decimals: 1,
        }
    }

    pub fn to_raw(&self) -> BigUint<M> {
        self.bu.clone()
    }
}
