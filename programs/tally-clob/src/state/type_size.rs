#![allow(dead_code)]

pub const BOOL_SIZE: usize = 1;
pub const CHAR_SIZE: usize = 4;
pub const DISCRIMINATOR_SIZE: usize = 8;
pub const ENUM_SIZE: usize = 1; // for data/field-less enums
pub const F32_SIZE: usize = 4;
pub const F64_SIZE: usize = 8;
pub const I64_SIZE: usize = 8;
pub const I128_SIZE: usize = 16;
pub const PUB_KEY_SIZE: usize = 32;
pub const U8_SIZE: usize = 1;
pub const U16_SIZE: usize = 2;
pub const U32_SIZE: usize = 4;
pub const U64_SIZE: usize = 8;
pub const U128_SIZE: usize = 16;

const OPTION_PREFIX_SIZE: usize = 1;
pub const fn option_size(element_size: usize) -> usize {
    OPTION_PREFIX_SIZE + element_size
}

const VEC_PREFIX_SIZE: usize = 4;
pub const fn vec_size(element_size: usize, length: usize) -> usize {
    VEC_PREFIX_SIZE + element_size * length
}

pub const fn string_size(str_len: usize) -> usize {
    vec_size(CHAR_SIZE, str_len)
}