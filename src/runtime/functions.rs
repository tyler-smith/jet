use crate::runtime::FN_NAME_CONTRACT_PREFIX;

pub fn mangle_contract_fn(address: &str) -> String {
    format!("{}{}", FN_NAME_CONTRACT_PREFIX, address)
}
