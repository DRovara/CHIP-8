//! Provides utility functions for the handling of numbers taken from Instructions. In particular, it supports the _Big Endian_ conversion of 4 and 8 bit integers.

#[deny(missing_docs)]

/// Converts two 4-bit unsigned integers into a _Big Endian_ encoded 8-bit unsigned integer.
/// 
/// The 4-bit integers are passed as `u8` because smaller datatypes are not supported. However, they may only hold a maximum value of 0xF, 
/// otherwise, an overflow may occur when running the function.
/// 
/// # Examples
/// ```
/// let most_significant_digit = 0xA;
/// let least_significant_digit = 0x8;
/// let big_endian_value = big_endian_4_2(most_significant_digit, least_significant_digit);
/// assert_eq!(big_endian_value, 0xA8);
/// ```
/// 
pub fn big_endian_4_2(n1: u8, n2: u8) -> u8 {
    0x10u8 * n1 + n2 as u8
}

/// Converts three 4-bit unsigned integers into a _Big Endian_ encoded 12-bit unsigned integer.
/// 
/// The 4-bit integers are passed as `u8` because smaller datatypes are not supported. However, they may only hold a maximum value of 0xF, 
/// otherwise, an overflow may occur when running the function. Similarly the result is expressed as a `u16` because `u12` is not supported by rust.
/// 
/// # Examples
/// ```
/// let most_significant_digit = 0xA;
/// let middle_digit = 0x2;
/// let least_significant_digit = 0x8;
/// let big_endian_value = big_endian_4_3(most_significant_digit, middle_digit, least_significant_digit);
/// assert_eq!(big_endian_value, 0xA28);
/// ```
/// 
pub fn big_endian_4_3(n1: u8, n2: u8, n3: u8) -> u16 {
    0x100u16 * n1 as u16 + 0x10u16 * n2 as u16 + n3 as u16
}

/// Converts two 8-bit unsigned integers into a _Big Endian_ encoded 16-bit unsigned integer.
/// 
/// # Examples
/// ```
/// let most_significant_byte = 0xA4;
/// let least_significant_byte = 0x8E;
/// let big_endian_value = big_endian_8_2(most_significant_byte, least_significant_byte);
/// assert_eq!(big_endian_value, 0xA48E);
/// ```
/// 
pub fn big_endian_8_2(n1: u8, n2: u8) -> u16 {
    0x100u16 * n1 as u16 + n2 as u16
}