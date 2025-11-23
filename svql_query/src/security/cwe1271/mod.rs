use crate::security::primitives::uninit_reg::{UninitReg, UninitRegEn};
use svql_macros::variant;

variant! {
    name: Cwe1271,
    variants: [
        (UninitRegEn, "uninit_reg_en", UninitRegEn),
        (UninitReg, "uninit_reg", UninitReg)
    ]
}
