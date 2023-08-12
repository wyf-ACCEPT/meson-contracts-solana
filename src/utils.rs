/// This file is for the functions in `MesonHelpers`.
/// Please note that:
///     1. Hexstrings such as `encodedSwap` in Solidity/Move is given by u8array in Rust

// use hex_string::HexString;

use arrayref::array_ref;
use solana_program::{
    keccak,
    pubkey::Pubkey, 
    secp256k1_recover::secp256k1_recover,
};

pub struct Utils {}

impl Utils {
    // Const variables
    pub const MESON_PROTOCOL_VERSION: u8 = 1;

    pub const SHORT_COIN_TYPE: [u8; 2] = [0x01, 0xf5]; // See https://github.com/satoshilabs/slips/blob/master/slip-0044.md, Solana is `800001f5` or `01f5`
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

    // Note that: We don't need the functions `is_encoded_valid` and `is_eth_addr`,
    //  because the param is length-fixed




    // encoded_swap: [u8; 32] = [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21]

    // Functions to obtain values from encoded
    // version: [{0x01}, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21]
    // (in solidity/move: `[01]001dcd6500c00000000000f677815c000000000000634dcb98027d0102ca21`)
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

    pub fn for_initial_chain(encoded_swap: [u8; 32]) {
        assert!(Self::in_chain_from(encoded_swap) == Self::SHORT_COIN_TYPE, "Swap in chain mismatch!");
    }

    pub fn for_target_chain(encoded_swap: [u8; 32]) {
        assert!(Self::out_chain_from(encoded_swap) == Self::SHORT_COIN_TYPE, "Swap out chain mismatch!");
    }

    pub fn get_swap_id(encoded_swap: [u8; 32], initiator: [u8; 20]) -> [u8; 32] {
        let mut buf = [0 as u8; 52];
        buf[..32].copy_from_slice(&encoded_swap);
        buf[32..].copy_from_slice(&initiator);
        keccak::hash(&buf).to_bytes()
    }

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

    // fee for lp: [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, {0x00, 0x00, 0x00, 0x00, 0x00}, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21] -> 0
    pub fn fee_for_lp(encoded_swap: [u8; 32]) -> u64 {
        Self::u8_5_to_u64(*array_ref![encoded_swap, 16, 5])
    }

    // expire timestamp: [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, {0x00, 0x63, 0x4d, 0xcb, 0x98}, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21] -> 1_666_042_776
    pub fn expire_ts_from(encoded_swap: [u8; 32]) -> u64 {
        Self::u8_5_to_u64(*array_ref![encoded_swap, 21, 5])
    }

    // target chain (slip44) -> [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, {0x02, 0x7d}, 0x01, 0x02, 0xca, 0x21]
    pub fn out_chain_from(encoded_swap: [u8; 32]) -> [u8; 2] {
        *array_ref![encoded_swap, 26, 2]
    }

    // target coin index: [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, {0x01}, 0x02, 0xca, 0x21]
    pub fn out_coin_index_from(encoded_swap: [u8; 32]) -> u8 {
        encoded_swap[28]
    }

    // source chain (slip44) -> [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, {0x02, 0xca}, 0x21]
    pub fn in_chain_from(encoded_swap: [u8; 32]) -> [u8; 2] {
        *array_ref![encoded_swap, 29, 2]
    }

    // source coin index: [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, {0x21}]
    pub fn in_coin_index_from(encoded_swap: [u8; 32]) -> u8 {
        encoded_swap[31]
    }




    pub fn check_request_signature(
        encoded_swap: [u8; 32],
        signature: [u8; 64],
        signer_eth_addr: [u8; 20]
    ) {
        let non_typed = Self::sign_non_typed(encoded_swap);
        let mut signing_data = Vec::new();

        if Self::in_chain_from(encoded_swap) == [0x00, 0xc3] {
            signing_data.extend_from_slice(if non_typed { &Self::TRON_SIGN_HEADER_33 } else { &Self::TRON_SIGN_HEADER });
            signing_data.extend_from_slice(&encoded_swap);
        }
        else if non_typed {
            signing_data.extend_from_slice(&Self::ETH_SIGN_HEADER);
            signing_data.extend_from_slice(&encoded_swap);
        } else {
            let msg_hash = keccak::hash(&encoded_swap);
            signing_data.extend_from_slice(&Self::REQUEST_TYPE);
            signing_data.extend_from_slice(&msg_hash.to_bytes());
        }
        println!("{:?}", signing_data);
        let digest = keccak::hash(&signing_data).to_bytes();
        println!("{:?}", digest);
        let recovered = Self::recover_eth_address(digest, signature);
        assert_eq!(recovered, signer_eth_addr, "Invalid signature!");
    }

    
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

    pub fn eth_address_from_solana_address(addr: Pubkey) -> [u8; 20] {
        let addr_bytes = addr.to_bytes();
        *array_ref![addr_bytes, 0, 20]
    }

    pub fn eth_address_from_pubkey(eth_pubkey: [u8; 64]) -> [u8; 20] {
        // Public key `eth_pubkey` should be uncompressed
        // Notice that Ethereum pubkey has an extra 0x04 prefix (specifies uncompressed)
        let hash = keccak::hash(&eth_pubkey).to_bytes();
        *array_ref![&hash, 12, 20]
    }

    pub fn recover_eth_address(digest: [u8; 32], signature: [u8; 64]) -> [u8; 20] {
        // EIP-2098: recovery_id is stored in first bit of sig.s
        let mut signature_split = [0 as u8; 64];
        signature_split.copy_from_slice(&signature);
        let first_bit_of_s = signature_split.get_mut(32).unwrap();
        let recovery_id = *first_bit_of_s >> 7;
        *first_bit_of_s = *first_bit_of_s & 0x7f;

        let pubkey = secp256k1_recover(&digest, recovery_id, &signature_split);
        match pubkey {
            Ok(eth_pubkey) => {
                Self::eth_address_from_pubkey(eth_pubkey.to_bytes())
            }
            Err(_error) => {
                [0 as u8; 20]           // Return 0x00 address if recover failed
            }
        }
    }
}


/// Note that tests are not allowed in the `impl` block.
#[cfg(test)]
mod tests {
    use solana_program::pubkey::Pubkey;

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

    #[test]
    pub fn test_get_swap_id() {
        let encoded_swap: [u8; 32] = [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21];
        let initiator: [u8; 20] = [0x2e, 0xf8, 0xa5, 0x1f, 0x8f, 0xf1, 0x29, 0xdb, 0xb8, 0x74, 0xa0, 0xef, 0xb0, 0x21, 0x70, 0x2f, 0x59, 0xc1, 0xb2, 0x11];
        
        let swap_id = Utils::get_swap_id(encoded_swap, initiator);
        assert_eq!(swap_id, [0xe3, 0xa8, 0x4c, 0xd4, 0x91, 0x2a, 0x01, 0x98, 0x9c, 0x6c, 0xd2, 0x4e, 0x41, 0xd3, 0xd9, 0x4b, 0xaf, 0x14, 0x32, 0x42, 0xfb, 0xf1, 0xda, 0x26, 0xeb, 0x7e, 0xac, 0x08, 0xc3, 0x47, 0xb6, 0x38]);
    }

    #[test]     // (Error warning in move analyzer)
    fn test_eth_address_from_aptos_address() {
        let solana_addr = Pubkey::new_from_array([0x01, 0x01, 0x5a, 0xce, 0x92, 0x0c, 0x71, 0x67, 0x94, 0x44, 0x59, 0x79, 0xbe, 0x68, 0xd4, 0x02, 0xd2, 0x8b, 0x28, 0x05, 0xb7, 0xbe, 0xaa, 0xe9, 0x35, 0xd7, 0xfe, 0x36, 0x9f, 0xa7, 0xcf, 0xa0]);

        let eth_addr = Utils::eth_address_from_solana_address(solana_addr);
        assert_eq!(eth_addr, [0x01, 0x01, 0x5a, 0xce, 0x92, 0x0c, 0x71, 0x67, 0x94, 0x44, 0x59, 0x79, 0xbe, 0x68, 0xd4, 0x02, 0xd2, 0x8b, 0x28, 0x05]);
    }

    #[test]
    fn test_eth_address_from_pubkey() {
        let eth_pubkey = [0x51, 0x39, 0xc6, 0xf9, 0x48, 0xe3, 0x8d, 0x3f, 0xfa, 0x36, 0xdf, 0x83, 0x60, 0x16, 0xae, 0xa0, 0x8f, 0x37, 0xa9, 0x40, 0xa9, 0x13, 0x23, 0xf2, 0xa7, 0x85, 0xd1, 0x7b, 0xe4, 0x35, 0x3e, 0x38, 0x2b, 0x48, 0x8d, 0x0c, 0x54, 0x3c, 0x50, 0x5e, 0xc4, 0x00, 0x46, 0xaf, 0xbb, 0x25, 0x43, 0xba, 0x6b, 0xb5, 0x6c, 0xa4, 0xe2, 0x6d, 0xc6, 0xab, 0xee, 0x13, 0xe9, 0xad, 0xd6, 0xb7, 0xe1, 0x89];

        let eth_addr = Utils::eth_address_from_pubkey(eth_pubkey);
        assert_eq!(eth_addr, [0x05, 0x2c, 0x77, 0x07, 0x09, 0x35, 0x34, 0x03, 0x5f, 0xc2, 0xed, 0x60, 0xde, 0x35, 0xe1, 0x1b, 0xeb, 0xb6, 0x48, 0x6b]);
    }

    #[test]
    fn test_recover_eth_address() {
        let eth_addr = Utils::recover_eth_address(
            [0xea, 0x83, 0xcd, 0xcd, 0xd0, 0x6b, 0xf6, 0x1e, 0x41, 0x40, 0x54, 0x11, 0x5a, 0x55, 0x1e, 0x23, 0x13, 0x37, 0x11, 0xd0, 0x50, 0x7d, 0xcb, 0xc0, 0x7a, 0x4b, 0xab, 0x7d, 0xc4, 0x58, 0x19, 0x35],
            [0x2b, 0xd0, 0x3a, 0x0d, 0x8e, 0xdf, 0xcb, 0xe8, 0x2e, 0x56, 0xff, 0xed, 0xe5, 0xa9, 0x4f, 0x49, 0x63, 0x5c, 0x80, 0x23, 0x64, 0x63, 0x0b, 0xc3, 0xbc, 0x9b, 0x17, 0xba, 0x85, 0xba, 0xad, 0xfa, 0xb8, 0xb7, 0x33, 0x43, 0x7f, 0x0a, 0xd8, 0x97, 0xaa, 0x24, 0x6d, 0x01, 0x11, 0x22, 0x57, 0x0c, 0x6c, 0x99, 0x43, 0xea, 0xd8, 0x62, 0x52, 0xd4, 0xf1, 0x69, 0x52, 0x49, 0x53, 0x80, 0xa3, 0x1e]
        );

        assert_eq!(eth_addr, [0x05, 0x2c, 0x77, 0x07, 0x09, 0x35, 0x34, 0x03, 0x5f, 0xc2, 0xed, 0x60, 0xde, 0x35, 0xe1, 0x1b, 0xeb, 0xb6, 0x48, 0x6b]);
    }

    #[test]
    fn test_check_request_signature() {
        let encoded_swap = [0x01, 0x00, 0x1d, 0xcd, 0x65, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x77, 0x81, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x4d, 0xcb, 0x98, 0x02, 0x7d, 0x01, 0x02, 0xca, 0x21];
        let signature = [0xb3, 0x18, 0x4c, 0x25, 0x7c, 0xf9, 0x73, 0x06, 0x92, 0x50, 0xee, 0xfd, 0x84, 0x9a, 0x74, 0xd2, 0x72, 0x50, 0xf8, 0x34, 0x3c, 0xbd, 0xa7, 0x61, 0x51, 0x91, 0x14, 0x9d, 0xd3, 0xc1, 0xb6, 0x1d, 0x5d, 0x4e, 0x2b, 0x5e, 0xcc, 0x76, 0xa5, 0x9b, 0xaa, 0xbf, 0x10, 0xa8, 0xd5, 0xd1, 0x16, 0xed, 0xb9, 0x5a, 0x5b, 0x20, 0x55, 0xb9, 0xb1, 0x9f, 0x71, 0x52, 0x40, 0x96, 0x97, 0x5b, 0x29, 0xc2];
        let eth_addr = [0x2e, 0xf8, 0xa5, 0x1f, 0x8f, 0xf1, 0x29, 0xdb, 0xb8, 0x74, 0xa0, 0xef, 0xb0, 0x21, 0x70, 0x2f, 0x59, 0xc1, 0xb2, 0x11];
        Utils::check_request_signature(encoded_swap, signature, eth_addr);
    }
}


// /// @title MesonHelpers
// /// @notice The class that provides helper functions for Meson protocol
// module Meson::MesonHelpers {

//     #[test]
//     #[expected_failure(abort_code=EINVALID_SIGNATURE)]
//     fn test_check_request_signature_error() {
//         let encoded_swap = x"01001dcd6500c00000000000f677815c000000000000634dcb98027d0102ca21";
//         let signature = x"b3184c257cf973069250eefd849a74d27250f8343cbda7615191149dd3c1b61d5d4e2b5ecc76a59baabf10a8d5d116edb95a5b2055b9b19f71524096975b29c3";
//         let eth_addr = x"2ef8a51f8ff129dbb874a0efb021702f59c1b211";
//         check_request_signature(encoded_swap, signature, eth_addr);
//     }


//     #[test]
//     fn test_check_release_signature() {
//         let encoded_swap = x"01001dcd6500c00000000000f677815c000000000000634dcb98027d0102ca21";
//         let recipient = x"01015ace920c716794445979be68d402d28b2805";
//         let signature = x"1205361aabc89e5b30592a2c95592ddc127050610efe92ff6455c5cfd43bdd825853edcf1fa72f10992b46721d17cb3191a85cefd2f8325b1ac59c7d498fa212";
//         let eth_addr = x"2ef8a51f8ff129dbb874a0efb021702f59c1b211";
//         check_release_signature(encoded_swap, recipient, signature, eth_addr);
//     }


// }
