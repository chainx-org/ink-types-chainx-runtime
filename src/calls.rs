// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::{AccountId, Balance, ChainXRuntimeTypes};
use ink_core::env::EnvTypes;
use scale::{Codec, Decode, Encode};
use sp_runtime::traits::Member;

/// Default runtime Call type, a subset of the runtime Call module variants
///
/// The codec indices of the  modules *MUST* match those in the concrete runtime.
#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(Clone, PartialEq, Eq))]
pub enum Call {
    #[codec(index = "5")]
    XAssets(XAssets<ChainXRuntimeTypes>),
    #[codec(index = "7")]
    XContracts(XContracts<ChainXRuntimeTypes>),
}

impl From<XAssets<ChainXRuntimeTypes>> for Call {
    fn from(assets_call: XAssets<ChainXRuntimeTypes>) -> Call {
        Call::XAssets(assets_call)
    }
}

impl From<XContracts<ChainXRuntimeTypes>> for Call {
    fn from(contracts_call: XContracts<ChainXRuntimeTypes>) -> Call {
        Call::XContracts(contracts_call)
    }
}

type Token = Vec<u8>;
type Memo = Vec<u8>;

/// Generic Balance Call, could be used with other runtimes
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub enum XAssets<T>
where
    T: EnvTypes,
    T::AccountId: Member + Codec,
{
    #[allow(non_camel_case_types)]
    #[codec(index = "3")]
    transfer(T::AccountId, Token, #[codec(compact)] T::Balance, Memo),
}

/// Construct a `Balances::transfer` call
pub fn transfer_balance(account: AccountId, token: Token, balance: Balance, memo: Memo) -> Call {
    XAssets::<ChainXRuntimeTypes>::transfer(account, token, balance, memo).into()
}

/// Generic Balance Call, could be used with other runtimes
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub enum XContracts<T>
where
    T: EnvTypes,
{
    #[allow(non_camel_case_types)]
    #[codec(index = "6")]
    convert_to_asset(T::AccountId, #[codec(compact)] T::Balance),
}

#[cfg(test)]
mod tests {
    use super::Call;
    use crate::{calls, ChainXRuntimeTypes};

    use chainx_runtime::{self, Runtime};
    use scale::{Decode, Encode};
    use sp_core::crypto::AccountId32;

    #[test]
    fn call_balance_transfer() {
        let balance = 10_000;

        let token = b"PCX".to_vec();
        let memo = b"memo".to_vec();

        let contract_address: AccountId32 = [1u8; 32].into();
        let contract_transfer = calls::XAssets::<ChainXRuntimeTypes>::transfer(
            contract_address.clone().into(),
            token.clone(),
            balance,
            memo.clone(),
        );
        let contract_call = Call::XAssets(contract_transfer);

        let srml_address: AccountId32 = contract_address.into();
        let srml_transfer = xassets::Call::<Runtime>::transfer(srml_address, token, balance, memo);
        let srml_call = chainx_runtime::Call::XAssets(srml_transfer);

        let contract_call_encoded = contract_call.encode();
        let srml_call_encoded = srml_call.encode();

        assert_eq!(srml_call_encoded, contract_call_encoded);

        let srml_call_decoded: chainx_runtime::Call =
            Decode::decode(&mut contract_call_encoded.as_slice())
                .expect("Balances transfer call decodes to srml type");
        let srml_call_encoded = srml_call_decoded.encode();
        let contract_call_decoded: Call = Decode::decode(&mut srml_call_encoded.as_slice())
            .expect("Balances transfer call decodes back to contract type");
        assert!(contract_call == contract_call_decoded);
    }
}
