// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

/// Single gas charge in the VM. Contains information about what gas was for, as well
/// as the amount of gas needed for computation and storage respectively.
pub struct GasCharge<'a> {
    pub name: &'a str,
    pub compute_gas: i64,
    pub storage_gas: i64,
}

impl<'a> GasCharge<'a> {
    pub fn new(name: &'a str, compute_gas: i64, storage_gas: i64) -> Self {
        Self {
            name,
            compute_gas,
            storage_gas,
        }
    }

    /// Calculates total gas charge based on compute and storage multipliers.
    pub fn total(&self) -> i64 {
        self.compute_gas + self.storage_gas
    }
}
