/// This file is for the functions in `MesonHelpers`.
/// Please note that:
///     1. Hexstrings such as `encodedSwap` in Solidity/Move is given by u8array in Rust

// use hex_string::HexString;

use arrayref::array_ref;

pub struct Utils {}

impl Utils {
    // Const variables
    pub const MESON_PROTOCOL_VERSION: u8 = 1;

    pub const SHORT_COIN_TYPE: [u8; 2] = [1, 245]; // See https://github.com/satoshilabs/slips/blob/master/slip-0044.md, Solana is `800001f5` or `01f5`
    pub const MAX_SWAP_AMOUNT: u64 = 100_000_000_000; // 100,000.000000 = 100k
    pub const MIN_BOND_TIME_PERIOD: u64 = 3600; // 1 hour
    pub const MAX_BOND_TIME_PERIOD: u64 = 7200; // 2 hours
    pub const LOCK_TIME_PERIOD: u64 = 1200; // 20 minutes

    pub const ETH_SIGN_HEADER: [u8; 28] = [
        25, 69, 116, 104, 101, 114, 101, 117, 109, 32, 83, 105, 103, 110, 101, 100, 32, 77, 101,
        115, 115, 97, 103, 101, 58, 10, 51, 50,
    ]; // That is "\x19Ethereum Signed Message:\n32"
    pub const ETH_SIGN_HEADER_52: [u8; 28] = [
        25, 69, 116, 104, 101, 114, 101, 117, 109, 32, 83, 105, 103, 110, 101, 100, 32, 77, 101,
        115, 115, 97, 103, 101, 58, 10, 53, 50,
    ]; // That is "\x19Ethereum Signed Message:\n52"
    pub const TRON_SIGN_HEADER: [u8; 25] = [
        25, 84, 82, 79, 78, 32, 83, 105, 103, 110, 101, 100, 32, 77, 101, 115, 115, 97, 103, 101,
        58, 10, 51, 50, 10,
    ]; // That is "\x19TRON Signed Message:\n32\n"
    pub const TRON_SIGN_HEADER_33: [u8; 25] = [
        25, 84, 82, 79, 78, 32, 83, 105, 103, 110, 101, 100, 32, 77, 101, 115, 115, 97, 103, 101,
        58, 10, 51, 51, 10,
    ]; // That is "\x19TRON Signed Message:\n33\n"
    pub const TRON_SIGN_HEADER_53: [u8; 25] = [
        25, 84, 82, 79, 78, 32, 83, 105, 103, 110, 101, 100, 32, 77, 101, 115, 115, 97, 103, 101,
        58, 10, 53, 51, 10,
    ]; // That is "\x19TRON Signed Message:\n53\n"

    // pub const REQUEST_TYPE: [u8; 39] = [
    //     98, 121, 116, 101, 115, 51, 50, 32, 83, 105, 103, 110, 32, 116, 111, 32, 114, 101, 113, 117,
    //     101, 115, 116, 32, 97, 32, 115, 119, 97, 112, 32, 111, 110, 32, 77, 101, 115, 111, 110,
    // ]; // That is "bytes32 Sign to request a swap on Meson"

    pub const REQUEST_TYPE: [u8; 49] = [
        98, 121, 116, 101, 115, 51, 50, 32, 83, 105, 103, 110, 32, 116, 111, 32, 114, 101, 113,
        117, 101, 115, 116, 32, 97, 32, 115, 119, 97, 112, 32, 111, 110, 32, 77, 101, 115, 111,
        110, 32, 40, 84, 101, 115, 116, 110, 101, 116, 41,
    ]; // That is "bytes32 Sign to request a swap on Meson (Testnet)"

    // pub const RELEASE_TYPE: [u8; 56] = [
    //     98, 121, 116, 101, 115, 51, 50, 32, 83, 105, 103, 110, 32, 116, 111, 32, 114, 101, 108, 101,
    //     97, 115, 101, 32, 97, 32, 115, 119, 97, 112, 32, 111, 110, 32, 77, 101, 115, 111, 110, 97, 100,
    //     100, 114, 101, 115, 115, 32, 82, 101, 99, 105, 112, 105, 101, 110, 116,
    // ]; // That is "bytes32 Sign to release a swap on Mesonaddress Recipient"

    pub const RELEASE_TYPE: [u8; 66] = [
        98, 121, 116, 101, 115, 51, 50, 32, 83, 105, 103, 110, 32, 116, 111, 32, 114, 101, 108,
        101, 97, 115, 101, 32, 97, 32, 115, 119, 97, 112, 32, 111, 110, 32, 77, 101, 115, 111, 110,
        32, 40, 84, 101, 115, 116, 110, 101, 116, 41, 97, 100, 100, 114, 101, 115, 115, 32, 82,
        101, 99, 105, 112, 105, 101, 110, 116,
    ]; // That is "bytes32 Sign to release a swap on Meson (Testnet)address Recipient"

    pub const RELEASE_TYPE_TRON: [u8; 85] = [
        98, 121, 116, 101, 115, 51, 50, 32, 83, 105, 103, 110, 32, 116, 111, 32, 114, 101, 108,
        101, 97, 115, 101, 32, 97, 32, 115, 119, 97, 112, 32, 111, 110, 32, 77, 101, 115, 111, 110,
        97, 100, 100, 114, 101, 115, 115, 32, 82, 101, 99, 105, 112, 105, 101, 110, 116, 32, 40,
        116, 114, 111, 110, 32, 97, 100, 100, 114, 101, 115, 115, 32, 105, 110, 32, 104, 101, 120,
        32, 102, 111, 114, 109, 97, 116, 41,
    ]; // That is "bytes32 Sign to release a swap on Mesonaddress Recipient (tron address in hex format)"

    pub fn get_MIN_BOND_TIME_PERIOD() -> u64 {
        Self::MIN_BOND_TIME_PERIOD
    }

    pub fn get_MAX_BOND_TIME_PERIOD() -> u64 {
        Self::MAX_BOND_TIME_PERIOD
    }

    pub fn get_LOCK_TIME_PERIOD() -> u64 {
        Self::LOCK_TIME_PERIOD
    }

    // Turn [u8; 5] to u64
    pub fn u8_5_to_u64(u8_5: [u8; 5]) -> u64 {
        let first = (u8_5[0] as u64) << 32;
        let rest = array_ref![u8_5, 1, 4];
        first + u32::from_be_bytes(*rest) as u64
    }

    // We don't need function: `is_encoded_valid` because it has fixed length

    // encoded_swap: [u8; 32] = [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21]

    // Functions to obtain values from encoded
    // version: [{0x01}, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21]
    //          (In solidity/move: `[01]001dcd6500c00000000000f677815c000000000000634dcb98027d0102ca21`)
    pub fn version_from(encoded_swap: [u8; 32]) -> u8 {
        encoded_swap[0]
    }

    // amount: [0x01, {0x00, 0x1d, 0xcd, 0x65, 0x00}, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21]
    pub fn amount_from(encoded_swap: [u8; 32]) -> u64 {
        Self::u8_5_to_u64(*array_ref![encoded_swap, 1, 5])
    }

    pub fn assert_amount_within_max(amount: u64) {
        assert!(amount <= Self::MAX_SWAP_AMOUNT, "Swap amount over max!");
    }

    pub fn match_protocol_version(encoded_swap: [u8; 32]) {
        assert!(Self::version_from(encoded_swap) == Self::MESON_PROTOCOL_VERSION, "Invalid encoded version!");
    }

    // pub fn for_initial_chain(encoded_swap: [u8; 32]) {
    //     assert!(in_chain_from(encoded_swap) == SHORT_COIN_TYPE, ESWAP_IN_CHAIN_MISMATCH);
    // }

    // pub fn for_target_chain(encoded_swap: [u8; 32]) {
    //     assert!(out_chain_from(encoded_swap) == SHORT_COIN_TYPE, ESWAP_OUT_CHAIN_MISMATCH);
    // }

    // service fee: Default to 0.1% of amount
    pub fn service_fee(encoded_swap: [u8; 32]) -> u64 {
        let amount = Self::amount_from(encoded_swap);
        amount * 10 / 10000
    }

    // salt & other infromation: [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, {0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c}, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21]
    pub fn salt_from(encoded_swap: [u8; 32]) -> [u8; 10] {
        *array_ref![encoded_swap, 6, 10]
    }

    // salt data: [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, {0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c}, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21]
    pub fn salt_data_from(encoded_swap: [u8; 32]) -> [u8; 9] {
        *array_ref![encoded_swap, 7, 9]
    }

    pub fn will_transfer_to_contract(encoded_swap: [u8; 32]) -> bool {
        encoded_swap[6] & 0x80 == 0x00
    }

    pub fn fee_waived(encoded_swap: [u8; 32]) -> bool {
        encoded_swap[6] & 0x40 == 0x40
    }

    pub fn sign_non_typed(encoded_swap: [u8; 32]) -> bool {
        encoded_swap[6] & 0x08 == 0x08
    }

    // fee for lp: [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, {0x00, 0x00, 0x00, 0x00, 0x00}, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21]
    pub fn fee_for_lp(encoded_swap: [u8; 32]) -> u64 {
        Self::u8_5_to_u64(*array_ref![encoded_swap, 16, 5])
    }

    // expire timestamp: [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, {0x00, 0x63, 0x4d, 0xcb, 0x98}, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21]
    pub fn expire_ts_from(encoded_swap: [u8; 32]) -> u64 {
        Self::u8_5_to_u64(*array_ref![encoded_swap, 21, 5])
    }

}


#[cfg(test)]
mod tests {
    use crate::utils::Utils;

    #[test]
    fn test_amount_from() {
        let encoded_swap = [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21];
        assert_eq!(Utils::amount_from(encoded_swap), 500_000_000);
        
        let encoded_swap = [0x01, 0x01, 0x2a, 0x05, 0xf2, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21];
        assert_eq!(Utils::amount_from(encoded_swap), 5_000_000_000);
    }

    #[test]
    fn test_fee_and_ts() {
        let encoded_swap = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21];
        assert_eq!(Utils::fee_for_lp(encoded_swap), 500_000_000);
        assert_eq!(Utils::expire_ts_from(encoded_swap), 1_666_042_776);
        
        let encoded_swap = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x01, 0x2a, 0x05, 0xf2, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21];
        assert_eq!(Utils::fee_for_lp(encoded_swap), 5_000_000_000);
    }
}


// /// @title MesonHelpers
// /// @notice The class that provides helper functions for Meson protocol
// module Meson::MesonHelpers {
//     use std::vector;
//     use std::option;
//     use std::bcs;
//     use std::aptos_hash;
//     use std::secp256k1;

//     const ETH_SIGN_HEADER: vector<u8> = b"\x19Ethereum Signed Message:\n32";

//     pub fn get_swap_id(encoded_swap: [u8; 32], initiator: vector<u8>) -> vector<u8> {
//         let buf = copy encoded_swap;
//         vector::append(&mut buf, initiator);
//         aptos_hash::keccak256(buf)
//     }

//     #[test]
//     pub fn test_get_swap_id() {
//         let swap_id = get_swap_id(
//             x"01001dcd6500c00000000000f677815c000000000000634dcb98027d0102ca21",
//             x"2ef8a51f8ff129dbb874a0efb021702f59c1b211"
//         );
//         assert!(swap_id == x"e3a84cd4912a01989c6cd24e41d3d94baf143242fbf1da26eb7eac08c347b638", 1);
//     }






//     // target chain (slip44) -> `01|001dcd6500|c00000000000f677815c|0000000000|00634dcb98|[027d]0102ca21`
//     pub fn out_chain_from(encoded_swap: [u8; 32]) -> vector<u8> {
//         vector[*vector::borrow(&encoded_swap, 26), *vector::borrow(&encoded_swap, 27)]
//     }

//     // target coin index: `01|001dcd6500|c00000000000f677815c|0000000000|00634dcb98|027d[01]02ca21`
//     pub fn out_coin_index_from(encoded_swap: [u8; 32]) -> u8 {
//         *vector::borrow(&encoded_swap, 28)
//     }

//     // source chain (slip44) -> `01|001dcd6500|c00000000000f677815c|0000000000|00634dcb98|027d01[02ca]21`
//     pub fn in_chain_from(encoded_swap: [u8; 32]) -> vector<u8> {
//         vector[*vector::borrow(&encoded_swap, 29), *vector::borrow(&encoded_swap, 30)]
//     }

//     // source coin index: `01|001dcd6500|c00000000000f677815c|0000000000|00634dcb98|027d0102ca[21]`
//     pub fn in_coin_index_from(encoded_swap: [u8; 32]) -> u8 {
//         *vector::borrow(&encoded_swap, 31)
//     }

//     pub fn check_request_signature(
//         encoded_swap: [u8; 32],
//         signature: vector<u8>,
//         signer_eth_addr: vector<u8>
//     ) {
//         is_eth_addr(signer_eth_addr);

//         let non_typed = sign_non_typed(encoded_swap);
//         let signing_data: vector<u8>;
//         if (in_chain_from(encoded_swap) == x"00c3") {
//             signing_data = if (non_typed) TRON_SIGN_HEADER_33 else TRON_SIGN_HEADER;
//             vector::append(&mut signing_data, encoded_swap);
//         } else if (non_typed) {
//             signing_data = ETH_SIGN_HEADER;
//             vector::append(&mut signing_data, encoded_swap);
//         } else {
//             let msg_hash = aptos_hash::keccak256(encoded_swap);
//             signing_data = aptos_hash::keccak256(REQUEST_TYPE);
//             vector::append(&mut signing_data, msg_hash);
//         };
//         let digest = aptos_hash::keccak256(signing_data);

//         let recovered = recover_eth_address(digest, signature);
//         assert!(recovered == signer_eth_addr, EINVALID_SIGNATURE);
//     }

//     #[test]
//     fn test_check_request_signature() {
//         let encoded_swap = x"01001dcd6500c00000000000f677815c000000000000634dcb98027d0102ca21";
//         let signature = x"b3184c257cf973069250eefd849a74d27250f8343cbda7615191149dd3c1b61d5d4e2b5ecc76a59baabf10a8d5d116edb95a5b2055b9b19f71524096975b29c2";
//         let eth_addr = x"2ef8a51f8ff129dbb874a0efb021702f59c1b211";
//         check_request_signature(encoded_swap, signature, eth_addr);
//     }

//     #[test]
//     #[expected_failure(abort_code=EINVALID_SIGNATURE)]
//     fn test_check_request_signature_error() {
//         let encoded_swap = x"01001dcd6500c00000000000f677815c000000000000634dcb98027d0102ca21";
//         let signature = x"b3184c257cf973069250eefd849a74d27250f8343cbda7615191149dd3c1b61d5d4e2b5ecc76a59baabf10a8d5d116edb95a5b2055b9b19f71524096975b29c3";
//         let eth_addr = x"2ef8a51f8ff129dbb874a0efb021702f59c1b211";
//         check_request_signature(encoded_swap, signature, eth_addr);
//     }

//     pub fn check_release_signature(
//         encoded_swap: [u8; 32],
//         recipient: vector<u8>,
//         signature: vector<u8>,
//         signer_eth_addr: vector<u8>,
//     ) {
//         is_eth_addr(signer_eth_addr);

//         let non_typed = sign_non_typed(encoded_swap);
//         let signing_data: vector<u8>;
//         if (in_chain_from(encoded_swap) == x"00c3") {
//             signing_data = if (non_typed) TRON_SIGN_HEADER_53 else TRON_SIGN_HEADER;
//             vector::append(&mut signing_data, encoded_swap);
//             vector::append(&mut signing_data, recipient);
//         } else if (non_typed) {
//             signing_data = ETH_SIGN_HEADER_52;
//             vector::append(&mut signing_data, encoded_swap);
//             vector::append(&mut signing_data, recipient);
//         } else {
//             let msg = copy encoded_swap;
//             vector::append(&mut msg, recipient);
//             let msg_hash = aptos_hash::keccak256(msg);
//             if (out_chain_from(encoded_swap) == x"00c3") {
//                 signing_data = aptos_hash::keccak256(RELEASE_TYPE_TRON);
//             } else {
//                 signing_data = aptos_hash::keccak256(RELEASE_TYPE);
//             };
//             vector::append(&mut signing_data, msg_hash);
//         };
//         let digest = aptos_hash::keccak256(signing_data);

//         let recovered = recover_eth_address(digest, signature);
//         assert!(recovered == signer_eth_addr, EINVALID_SIGNATURE);
//     }

//     #[test]
//     fn test_check_release_signature() {
//         let encoded_swap = x"01001dcd6500c00000000000f677815c000000000000634dcb98027d0102ca21";
//         let recipient = x"01015ace920c716794445979be68d402d28b2805";
//         let signature = x"1205361aabc89e5b30592a2c95592ddc127050610efe92ff6455c5cfd43bdd825853edcf1fa72f10992b46721d17cb3191a85cefd2f8325b1ac59c7d498fa212";
//         let eth_addr = x"2ef8a51f8ff129dbb874a0efb021702f59c1b211";
//         check_release_signature(encoded_swap, recipient, signature, eth_addr);
//     }

//     pub fn is_eth_addr(addr: vector<u8>) {
//         assert!(vector::length(&addr) == 20, EINVALID_ETH_ADDRESS);
//     }

//     pub fn eth_address_from_aptos_address(addr: address) -> vector<u8> {
//         let addr_bytes = bcs::to_bytes(&addr);
//         let eth_addr = vector::empty<u8>();
//         let i = 0;
//         while (i < 20) {
//             vector::push_back(&mut eth_addr, *vector::borrow(&addr_bytes, i));
//             i = i + 1;
//         };
//         eth_addr
//     }

//     // #[test]     // (Error warning in move analyzer)
//     // fn test_eth_address_from_aptos_address() {
//     //     let aptos_addr = @0x01015ace920c716794445979be68d402d28b2805b7beaae935d7fe369fa7cfa0;
//     //     let eth_addr = eth_address_from_aptos_address(aptos_addr);
//     //     assert!(eth_addr == x"01015ace920c716794445979be68d402d28b2805", 1);
//     // }

//     pub fn eth_address_from_pubkey(pk: vector<u8>) -> vector<u8> {
//         // Public key `pk` should be uncompressed
//         // Notice that Ethereum pubkey has an extra 0x04 prefix (specifies uncompressed)
//         assert!(vector::length(&pk) == 64, EINVALID_PUBLIC_KEY);
//         let hash = aptos_hash::keccak256(pk);
//         let eth_addr = vector::empty<u8>();
//         let i = 12;
//         while (i < 32) {
//             vector::push_back(&mut eth_addr, *vector::borrow(&hash, i));
//             i = i + 1;
//         };
//         eth_addr
//     }

//     #[test]
//     fn test_eth_address_from_pubkey() {
//         let pk = x"5139c6f948e38d3ffa36df836016aea08f37a940a91323f2a785d17be4353e382b488d0c543c505ec40046afbb2543ba6bb56ca4e26dc6abee13e9add6b7e189";
//         let eth_addr = eth_address_from_pubkey(pk);
//         assert!(eth_addr == x"052c7707093534035fc2ed60de35e11bebb6486b", 1);
//     }

//     pub fn recover_eth_address(digest: vector<u8>, signature: vector<u8>) -> vector<u8> {
//         // EIP-2098: recovery_id is stored in first bit of sig.s
//         let first_bit_of_s = vector::borrow_mut(&mut signature, 32);
//         let recovery_id = *first_bit_of_s >> 7;
//         *first_bit_of_s = *first_bit_of_s & 0x7f;

//         let ecdsa_sig = secp256k1::ecdsa_signature_from_bytes(signature);
//         let pk = secp256k1::ecdsa_recover(digest, recovery_id, &ecdsa_sig);
//         if (option::is_some(&pk)) {
//             let extracted = option::extract(&mut pk);
//             let raw_pk = secp256k1::ecdsa_raw_pub_key_to_bytes(&extracted);
//             eth_address_from_pubkey(raw_pk)
//         } else {
//             vector::empty<u8>()
//         }
//     }

//     #[test]
//     fn test_recover_eth_address() {
//         let eth_addr = recover_eth_address(
//             x"ea83cdcdd06bf61e414054115a551e23133711d0507dcbc07a4bab7dc4581935",
//             x"2bd03a0d8edfcbe82e56ffede5a94f49635c802364630bc3bc9b17ba85baadfab8b733437f0ad897aa246d011122570c6c9943ead86252d4f16952495380a31e"
//         );
//         assert!(eth_addr == x"052c7707093534035fc2ed60de35e11bebb6486b", 1);
//     }
// }
