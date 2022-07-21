use anyhow::{anyhow, Result};
use std::ops::{BitAnd, BitOr, Shl, Shr};

/// Utilities for getting, setting, and counting bits within a byte or array of bytes.
#[derive(Debug, PartialEq)]
pub enum Bit {
    Zero,
    One,
}

impl Bit {
    /// Get the i-th bit of a byte array where the 0-th bit is the most significant bit.
    ///
    /// # Example
    ///
    /// ```
    /// let one: Bit::One = Bit::get_bit(&[0b10000000_u8, 0b00000000_u8], 0).unwrap();
    /// let one: Bit::One = Bit::get_bit(&[0b01000000_u8, 0b00000000_u8], 1).unwrap();
    /// let one: Bit::One = Bit::get_bit(&[0b00000000_u8, 0b00000001_u8], 15).unwrap();
    /// ```
    pub fn get_bit(v: &[u8], i: u32) -> Result<Bit> {
        if v.len() == 0 || i >= (v.len() * 8) as u32 {
            Err(anyhow!(
                "IllegalArgumentError: bytes.length = {}; i = {}",
                v.len(),
                i
            ))
        } else {
            let b = unsafe { v.get_unchecked((i / 8) as usize) };

            Bit::get_bit_u8(b, i % 8)
        }
    }

    /// Get the i-th bit of a byte where the 0-th bit is the most significant bit,
    /// and the 7-th bit is the least significant bit.
    ///
    /// # Example
    ///
    /// ```
    /// let zero: Bit::Zero = Bit::get_bit_u8(&0b10000000_u8, 7).unwrap();
    /// let zero: Bit::Zero = Bit::get_bit_u8(&0b00100000_u8, 1).unwrap();
    /// let one: Bit::One = Bit::get_bit_u8(&0b10000000_u8, 0).unwrap();
    /// let one: Bit::One = Bit::get_bit_u8(&0b01000000_u8, 1).unwrap();
    /// ```
    fn get_bit_u8(v: &u8, i: u32) -> Result<Bit> {
        if i >= 8 {
            Err(anyhow!("IllegalArgumentError: index {} out of bounds", i))
        } else {
            // return ((v >> (7 - i)) & 1) == 0 ? Bit.ZERO : Bit.ONE;
            let v = v.shr(7 - i).bitand(1_u8);
            if v.eq(&0) {
                Ok(Bit::Zero)
            } else {
                Ok(Bit::One)
            }
        }
    }

    /// Set the i-th bit of a byte array where the 0-th bit is the most significant bit of the first byte in array.
    ///
    /// # Example
    ///
    /// ```
    /// let mut buf = [0b00000000_u8, 0b00000000_u8];
    /// Bit::set_bit(&mut buf, 0, Bit::One);  // [0b10000000_u8, 0b00000000_u8]
    /// Bit::set_bit(&mut buf, 1, Bit::One);  // [0b11000000_u8, 0b00000000_u8]
    /// Bit::set_bit(&mut buf, 2, Bit::One);  // [0b11100000_u8, 0b00000000_u8]
    /// Bit::set_bit(&mut buf, 15, Bit::One); // [0b11100000_u8, 0b00000001_u8]
    /// ```
    pub fn set_bit(v: &mut [u8], i: u32, bit: Bit) -> Result<()> {
        let b = unsafe { v.get_unchecked_mut((i / 8) as usize) };
        *b = Bit::set_bit_u8(b, i % 8, bit)?;
        Ok(())
    }

    /// Set the i-th bit of a byte where the 0-th bit is the most significant bit,
    /// and the 7-th bit is the least significant bit.
    ///
    /// # Example
    ///
    /// ```
    /// let v = Bit::set_bit_u8(0b00000000_u8, 0, Bit::One).unwrap(); // 0b10000000_u8
    /// let v = Bit::set_bit_u8(0b00000000_u8, 1, Bit::One).unwrap(); // 0b01000000_u8
    /// let v = Bit::set_bit_u8(0b00000000_u8, 2, Bit::One).unwrap(); // 0b00100000_u8
    /// ```
    fn set_bit_u8(v: &u8, i: u32, bit: Bit) -> Result<u8> {
        if i >= 8 {
            Err(anyhow!("IllegalArgumentError: index {} out of bounds", i))
        } else {
            // 0x00000001 << (7 - i)
            let mask = 0b00000001_u8.shl(7 - i);

            // println!(
            //     "mask:{:#08b} re_mask:{:#08b} re_mask2:{:#08b} zero:{:#08b} one:{:#08b}",
            //     mask,
            //     mask.reverse_bits(),
            //     !mask,
            //     v.bitand(mask.reverse_bits()),
            //     v.bitor(mask)
            // );

            match bit {
                // v & ~mask
                // Rust bitwise Not Operation: https://stackoverflow.com/questions/38896155/what-is-the-bitwise-not-operator-in-rust
                Bit::Zero => Ok(v.bitand(!mask)),
                // v | mask
                Bit::One => Ok(v.bitor(mask)),
            }
        }
    }

    /// Counts the number of 1 bit flag in byte array.
    ///
    /// # Example
    ///
    /// ```
    /// let cnt = Bit::count_ones(&[0b00001010_u8, 0b11111101_u8]); // 9
    /// let cnt = Bit::count_ones(&[0b11111101_u8, 0b11111101_u8]); // 14
    /// ```
    pub fn count_ones(v: &[u8]) -> u32 {
        v.iter().map(|b| b.count_ones()).sum()
    }

    /// Counts the number of 1 bit flag in byte.
    ///
    /// # Example
    ///
    /// ```
    /// let cnt = Bit::count_ones_u8(&0b00001010_u8); // 2
    /// let cnt = Bit::count_ones_u8(&0b11111101_u8); // 7
    /// ```
    fn count_ones_u8(v: &u8) -> u32 {
        v.count_ones()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How to test a function which returns Result
    /// https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html
    #[test]
    fn test_get_bit_on_byte() -> Result<()> {
        let v = 0b01101011_u8;

        assert_eq!(Bit::Zero, Bit::get_bit_u8(&v, 0)?);
        assert_eq!(Bit::One, Bit::get_bit_u8(&v, 1)?);
        assert_eq!(Bit::One, Bit::get_bit_u8(&v, 2)?);
        assert_eq!(Bit::Zero, Bit::get_bit_u8(&v, 3)?);
        assert_eq!(Bit::One, Bit::get_bit_u8(&v, 4)?);
        assert_eq!(Bit::Zero, Bit::get_bit_u8(&v, 5)?);
        assert_eq!(Bit::One, Bit::get_bit_u8(&v, 6)?);
        assert_eq!(Bit::One, Bit::get_bit_u8(&v, 7)?);

        Ok(())
    }

    #[test]
    fn test_get_bit_on_bytes() -> Result<()> {
        let v: [u8; 2] = [0b01101011, 0b01001101];

        assert_eq!(Bit::Zero, Bit::get_bit(&v, 0)?);
        assert_eq!(Bit::One, Bit::get_bit(&v, 1)?);
        assert_eq!(Bit::One, Bit::get_bit(&v, 2)?);
        assert_eq!(Bit::Zero, Bit::get_bit(&v, 3)?);
        assert_eq!(Bit::One, Bit::get_bit(&v, 4)?);
        assert_eq!(Bit::Zero, Bit::get_bit(&v, 5)?);
        assert_eq!(Bit::One, Bit::get_bit(&v, 6)?);
        assert_eq!(Bit::One, Bit::get_bit(&v, 7)?);

        assert_eq!(Bit::Zero, Bit::get_bit(&v, 8)?);
        assert_eq!(Bit::One, Bit::get_bit(&v, 9)?);
        assert_eq!(Bit::Zero, Bit::get_bit(&v, 10)?);
        assert_eq!(Bit::Zero, Bit::get_bit(&v, 11)?);
        assert_eq!(Bit::One, Bit::get_bit(&v, 12)?);
        assert_eq!(Bit::One, Bit::get_bit(&v, 13)?);
        assert_eq!(Bit::Zero, Bit::get_bit(&v, 14)?);
        assert_eq!(Bit::One, Bit::get_bit(&v, 15)?);

        Ok(())
    }

    #[test]
    fn test_set_bit_on_byte() -> Result<()> {
        let v = 0b00000000_u8;
        assert_eq!(0b10000000_u8, Bit::set_bit_u8(&v, 0, Bit::One)?);
        assert_eq!(0b01000000_u8, Bit::set_bit_u8(&v, 1, Bit::One)?);
        assert_eq!(0b00100000_u8, Bit::set_bit_u8(&v, 2, Bit::One)?);
        assert_eq!(0b00010000_u8, Bit::set_bit_u8(&v, 3, Bit::One)?);
        assert_eq!(0b00001000_u8, Bit::set_bit_u8(&v, 4, Bit::One)?);
        assert_eq!(0b00000100_u8, Bit::set_bit_u8(&v, 5, Bit::One)?);
        assert_eq!(0b00000010_u8, Bit::set_bit_u8(&v, 6, Bit::One)?);
        assert_eq!(0b00000001_u8, Bit::set_bit_u8(&v, 7, Bit::One)?);

        let v = 0b11111111_u8;
        assert_eq!(0b01111111_u8, Bit::set_bit_u8(&v, 0, Bit::Zero)?);
        assert_eq!(0b10111111_u8, Bit::set_bit_u8(&v, 1, Bit::Zero)?);
        assert_eq!(0b11011111_u8, Bit::set_bit_u8(&v, 2, Bit::Zero)?);
        assert_eq!(0b11101111_u8, Bit::set_bit_u8(&v, 3, Bit::Zero)?);
        assert_eq!(0b11110111_u8, Bit::set_bit_u8(&v, 4, Bit::Zero)?);
        assert_eq!(0b11111011_u8, Bit::set_bit_u8(&v, 5, Bit::Zero)?);
        assert_eq!(0b11111101_u8, Bit::set_bit_u8(&v, 6, Bit::Zero)?);
        assert_eq!(0b11111110_u8, Bit::set_bit_u8(&v, 7, Bit::Zero)?);

        Ok(())
    }

    #[test]
    fn test_set_bit_on_bytes() -> Result<()> {
        let mut v: [u8; 2] = [0b00000000, 0b00000000];

        let expected_one: [[u8; 2]; 16] = [
            [0b10000000_u8, 0b00000000_u8],
            [0b11000000_u8, 0b00000000_u8],
            [0b11100000_u8, 0b00000000_u8],
            [0b11110000_u8, 0b00000000_u8],
            [0b11111000_u8, 0b00000000_u8],
            [0b11111100_u8, 0b00000000_u8],
            [0b11111110_u8, 0b00000000_u8],
            [0b11111111_u8, 0b00000000_u8],
            [0b11111111_u8, 0b10000000_u8],
            [0b11111111_u8, 0b11000000_u8],
            [0b11111111_u8, 0b11100000_u8],
            [0b11111111_u8, 0b11110000_u8],
            [0b11111111_u8, 0b11111000_u8],
            [0b11111111_u8, 0b11111100_u8],
            [0b11111111_u8, 0b11111110_u8],
            [0b11111111_u8, 0b11111111_u8],
        ];

        for i in 0..16 {
            Bit::set_bit(&mut v, i, Bit::One)?;
            assert_eq!(expected_one[i as usize], v);
        }

        let expected_zero: [[u8; 2]; 16] = [
            [0b01111111_u8, 0b11111111_u8],
            [0b00111111_u8, 0b11111111_u8],
            [0b00011111_u8, 0b11111111_u8],
            [0b00001111_u8, 0b11111111_u8],
            [0b00000111_u8, 0b11111111_u8],
            [0b00000011_u8, 0b11111111_u8],
            [0b00000001_u8, 0b11111111_u8],
            [0b00000000_u8, 0b11111111_u8],
            [0b00000000_u8, 0b01111111_u8],
            [0b00000000_u8, 0b00111111_u8],
            [0b00000000_u8, 0b00011111_u8],
            [0b00000000_u8, 0b00001111_u8],
            [0b00000000_u8, 0b00000111_u8],
            [0b00000000_u8, 0b00000011_u8],
            [0b00000000_u8, 0b00000001_u8],
            [0b00000000_u8, 0b00000000_u8],
        ];

        for i in 0..16 {
            Bit::set_bit(&mut v, i, Bit::Zero)?;
            assert_eq!(expected_zero[i as usize], v);
        }

        Ok(())
    }

    #[test]
    fn test_count_ones_on_byte() {
        assert_eq!(Bit::count_ones_u8(&0b11111111_u8), 8);
        assert_eq!(Bit::count_ones_u8(&0b01111111_u8), 7);
        assert_eq!(Bit::count_ones_u8(&0b00111111_u8), 6);
        assert_eq!(Bit::count_ones_u8(&0b00011111_u8), 5);
        assert_eq!(Bit::count_ones_u8(&0b00001111_u8), 4);
        assert_eq!(Bit::count_ones_u8(&0b00000111_u8), 3);
        assert_eq!(Bit::count_ones_u8(&0b00000011_u8), 2);
        assert_eq!(Bit::count_ones_u8(&0b00000001_u8), 1);
        assert_eq!(Bit::count_ones_u8(&0b00000000_u8), 0);
    }

    #[test]
    fn test_count_ones_on_bytes() {
        let v: [[u8; 2]; 16] = [
            [0b00000000_u8, 0b00000000_u8],
            [0b00000000_u8, 0b00000001_u8],
            [0b00000000_u8, 0b00000011_u8],
            [0b00000000_u8, 0b00000111_u8],
            [0b00000000_u8, 0b00001111_u8],
            [0b00000000_u8, 0b00011111_u8],
            [0b00000000_u8, 0b00111111_u8],
            [0b00000000_u8, 0b01111111_u8],
            [0b00000000_u8, 0b11111111_u8],
            [0b00000001_u8, 0b11111111_u8],
            [0b00000011_u8, 0b11111111_u8],
            [0b00000111_u8, 0b11111111_u8],
            [0b00001111_u8, 0b11111111_u8],
            [0b00011111_u8, 0b11111111_u8],
            [0b00111111_u8, 0b11111111_u8],
            [0b01111111_u8, 0b11111111_u8],
        ];
        for i in 0..=15 {
            assert_eq!(i as u32, Bit::count_ones(&v[i]))
        }
    }
}
