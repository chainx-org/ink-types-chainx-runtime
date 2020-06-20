#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

mod crypto {
    /// Do a Blake2 256-bit hash and place result in `dest`.
    pub fn blake2_256_into(data: &[u8], dest: &mut [u8; 32]) {
        dest.copy_from_slice(blake2_rfc::blake2b::blake2b(32, &[], data).as_bytes());
    }

    /// Do a Blake2 256-bit hash and return result.
    pub fn blake2_256(data: &[u8]) -> [u8; 32] {
        let mut r = [0; 32];
        blake2_256_into(data, &mut r);
        r
    }
}

#[ink::contract(version = "0.1.0", env = ChainXRuntimeTypes)]
mod calls {
    use super::crypto;
    use ink_core::env;
    use ink_prelude::collections::BTreeMap;
    use ink_prelude::format;
    use ink_types_node_runtime::{calls as runtime_calls, ChainXRuntimeTypes};
    use scale::{Decode, Encode, KeyedVec};

    #[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Copy, Encode, Decode)]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    pub enum AssetType {
        Free,
        ReservedStaking,
        ReservedStakingRevocation,
        ReservedWithdrawal,
        ReservedDexSpot,
        ReservedDexFuture,
        ReservedCurrency,
        ReservedXRC20,
        GasPayment,
    }

    /// This simple dummy contract dispatches substrate runtime calls
    #[ink(storage)]
    struct Calls {}

    impl Calls {
        #[ink(constructor)]
        fn new(&mut self) {}

        /// Dispatches a `transfer` call to the Balances srml module
        #[ink(message)]
        fn pcx_transfer(&self, dest: AccountId, value: Balance) {
            // create the Balances::transfer Call
            let transfer_call =
                runtime_calls::asset_transfer(dest, b"PCX".to_vec(), value, b"memo".to_vec());
            // dispatch the call to the runtime
            let result = self.env().invoke_runtime(&transfer_call);

            // report result to console
            // NOTE: println should only be used on a development chain)
            env::println(&format!(
                "Balance transfer invoke_runtime result {:?}",
                result
            ));
        }

        /// Returns the account balance, read directly from runtime storage
        #[ink(message)]
        fn get_asset_balance(&self, account: AccountId) -> BTreeMap<AssetType, u64> {
            const BALANCE_OF: &[u8] = b"XAssets AssetBalance";
            let pcx_balance = (account, b"PCX".to_vec());
            let key = crypto::blake2_256(&pcx_balance.to_keyed_vec(BALANCE_OF));
            let result = self
                .env()
                .get_runtime_storage::<BTreeMap<AssetType, u64>>(&key[..]);
            result.map(|x| x.unwrap_or_default()).unwrap_or_default()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use sp_keyring::AccountKeyring;

        #[test]
        fn dispatches_balances_call() {
            let calls = Calls::new();
            let alice = AccountId::from(AccountKeyring::Alice.to_account_id());
            // assert_eq!(calls.env().dispatched_calls().into_iter().count(), 0);
            calls.pcx_transfer(alice, 10000);
            // assert_eq!(calls.env().dispatched_calls().into_iter().count(), 1);
        }
    }
}
