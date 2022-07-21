use crate::common::Bit;
use anyhow::{anyhow, Result};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use std::io::Write;

/// An enum to represent the byte order of the `ByteBuffer`
#[derive(Debug, Copy, Clone)]
pub enum Endian {
    Big,
    Little,
}

macro_rules! read_number {
    ($self:ident, $name:ident, $offset:expr) => {{
        $self.flush_bit();

        if $self.r_pos + $offset > $self.data.len() {
            return Err(anyhow!("Could not read enough bytes from buffer"));
        }

        let range = $self.r_pos..($self.r_pos + $offset);
        $self.r_pos += $offset;

        Ok(match $self.endian {
            Endian::Big => BigEndian::$name(&$self.data[range]),
            Endian::Little => LittleEndian::$name(&$self.data[range]),
        })
    }};
}

/// A byte buffer object specifically turned to easily read and write binary values
pub struct ByteBuffer {
    /// byte array container
    data: Vec<u8>,
    /// current writing cursor
    w_pos: usize,
    /// current reading cursor
    r_pos: usize,
    /// current bitwise writing cursor
    w_bit: usize,
    /// current bitwise reading cursor
    r_bit: usize,
    /// byte order representation
    endian: Endian,
}

impl ByteBuffer {
    /// Construct an empty byte buffer
    pub fn new() -> Self {
        Self {
            data: vec![],
            w_pos: 0,
            r_pos: 0,
            w_bit: 0,
            r_bit: 0,
            endian: Endian::Big,
        }
    }

    /// Construct a new ByteBuffer filled with a specified byte array
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut buf = ByteBuffer::new();
        buf.write_bytes(bytes);
        buf
    }

    /// Returns the buffer size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the ByteBuffer contains no elements.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear the buffer and reinitialize the reading and writing cursor
    pub fn clear(&mut self) {
        self.data.clear();
        self.w_pos = 0;
        self.r_pos = 0;
    }

    /// Change the buffer size to input size
    ///
    /// _Note_: You can't shrink a buffer with this method
    pub fn resize(&mut self, size: usize) {
        let diff = size - self.data.len();
        if diff > 0 {
            let iter = std::iter::repeat(0).take(diff);
            self.data.extend(iter)
        }
    }

    /// Set the byte order of the buffer
    ///
    /// _Note_: By default, the buffer uses `Endian::Big` order
    pub fn set_endian(&mut self, endian: Endian) {
        self.endian = endian;
    }

    /// Returns the current  byte order of this buffer
    pub fn endian(&self) -> Endian {
        self.endian
    }

    /// Append a byte array to the buffer. The buffer is automatically extended if needed
    /// _Note_: This method resets the read and write cursor for bitwise reading
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    /// let mut buf = ByteBuffer::new();
    /// buf.write_bytes(&vec![0x1, 0xFF, 0x45]); // buffer contains [0x1, 0xFF, 0x45]
    /// ```
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.flush_bit();

        let size = bytes.len() + self.w_pos;

        // automatically extend buffer size if need
        if size > self.data.len() {
            self.resize(size);
        }

        for byte in bytes {
            self.data[self.w_pos] = *byte;
            self.w_pos += 1;
        }
    }

    /// Append a byte(8 bits) to the buffer
    /// _Note_: This method resets the read and write cursor for bitwise reading
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    ///
    /// let mut buf = ByteBuffer::new();
    /// buf.write_u8(1_u8); // buffer contains [0x1]
    /// ```
    pub fn write_u8(&mut self, val: u8) {
        self.write_bytes(&[val]);
    }

    /// Same as `write_u8()` method but for signed values.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn write_i8(&mut self, val: i8) {
        self.write_u8(val as u8);
    }

    /// Append a word(16 bits) to the buffer
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    ///
    /// let mut buf = ByteBuffer::new();
    /// buf.write_u16(1_u16); // buffer contains [0x00, 0x01] if little endian
    /// ```
    pub fn write_u16(&mut self, val: u16) {
        let mut buf = [0; 2];

        match self.endian {
            Endian::Big => BigEndian::write_u16(&mut buf, val),
            Endian::Little => LittleEndian::write_u16(&mut buf, val),
        }

        self.write_bytes(&buf);
    }

    /// Same as `write_u16()` method but for signed values.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn write_i16(&mut self, val: i16) {
        self.write_u16(val as u16);
    }

    /// Append a double word(32 bits) to the buffer
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    ///
    /// let mut buf = ByteBuffer::new();
    /// buf.write_u32(1_u32); // buffer contains [0x00, 0x00, 0x00, 0x01] if little endian
    /// ```
    pub fn write_u32(&mut self, val: u32) {
        let mut buf = [0; 4];

        match self.endian {
            Endian::Big => BigEndian::write_u32(&mut buf, val),
            Endian::Little => LittleEndian::write_u32(&mut buf, val),
        }

        self.write_bytes(&buf);
    }

    /// Same as `write_u32()` method but for signed values
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn write_i32(&mut self, val: i32) {
        self.write_u32(val as u32);
    }

    /// Append a quad word(64 bits) to the buffer
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    ///
    /// let mut buf = ByteBuffer::new();
    /// buf.write_u64(1_u64); // buffer contains [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01] if little endian
    /// ```
    pub fn write_u64(&mut self, val: u64) {
        let mut buf = [0; 8];

        match self.endian {
            Endian::Big => BigEndian::write_u64(&mut buf, val),
            Endian::Little => LittleEndian::write_u64(&mut buf, val),
        }

        self.write_bytes(&buf);
    }

    /// Same as `write_u64()` method for signed values.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn write_i64(&mut self, val: i64) {
        self.write_u64(val as u64);
    }

    /// Append a 32 bits floating point number to the buffer.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    ///
    /// let mut buf = ByteBuffer::new();
    /// buf.write_f32(0.1_f32);
    /// ```
    pub fn write_f32(&mut self, val: f32) {
        let mut buf = [0; 4];

        match self.endian {
            Endian::Big => BigEndian::write_f32(&mut buf, val),
            Endian::Little => LittleEndian::write_f32(&mut buf, val),
        }

        self.write_bytes(&buf);
    }

    /// Append a 64 bits floating pointing number to the buffer.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    ///
    /// let mut buf = ByteBuffer::new();
    /// buf.write_f64(0.1_f64);
    /// ```
    pub fn write_f64(&mut self, val: f64) {
        let mut buf = [0; 8];

        match self.endian {
            Endian::Big => BigEndian::write_f64(&mut buf, val),
            Endian::Little => LittleEndian::write_f64(&mut buf, val),
        }

        self.write_bytes(&buf);
    }

    /// Append a string to the buffer.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// *Format* The format is `(u32)size + size * (u8)characters`
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    ///
    /// let mut buf = ByteBuffer::new();
    /// buf.write_string("example");
    /// ```
    pub fn write_string(&mut self, val: &str) {
        self.write_u32(val.len() as u32);
        self.write_bytes(val.as_bytes());
    }

    /// Read a defined amount of raw bytes, or Return an error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>> {
        self.flush_bit();

        if self.r_pos + size > self.data.len() {
            return Err(anyhow!("Could not read enough bytes from buffer"));
        }

        let range = self.r_pos..(self.r_pos + size);
        let mut res = Vec::<u8>::new();
        res.write_all(&self.data[range])?;
        self.r_pos += size;
        Ok(res)
    }

    /// Read one byte, or return an error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    ///
    /// let mut buf = ByteBuffer::from_bytes(&vec![0x1]);
    /// let value = buf.read_u8().unwrap(); // value contains 1
    /// ```
    pub fn read_u8(&mut self) -> Result<u8> {
        self.flush_bit();

        if self.r_pos >= self.data.len() {
            return Err(anyhow!("Could not read enough bytes from buffer"));
        }

        let pos = self.r_pos;
        self.r_pos += 1;
        Ok(self.data[pos])
    }

    /// Same as `read_u8()` method but for signed values
    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    /// Read a 2-bytes long value, or return an error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    ///
    /// let mut buf = ByteBuffer::from_bytes(&vec![0x0, 0x1]);
    /// let value = buffer.read_u16().unwrap(); // value contains 1
    /// ```
    pub fn read_u16(&mut self) -> Result<u16> {
        read_number!(self, read_u16, 2)
    }

    /// Same as `read_u16()` method but for signed values
    pub fn read_i16(&mut self) -> Result<i16> {
        Ok(self.read_u16()? as i16)
    }

    /// Read a 4-bytes long value, or return an error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    ///
    /// let mut buf = ByteBuffer::from_bytes(&vec![0x0, 0x0, 0x0, 0x1]);
    /// let value = buf.read_u32().unwrap(); // value contains 1
    /// ```
    pub fn read_u32(&mut self) -> Result<u32> {
        read_number!(self, read_u32, 4)
    }

    /// Same as `read_u32()` method but for signed values
    pub fn read_i32(&mut self) -> Result<i32> {
        Ok(self.read_u32()? as i32)
    }

    /// Read a 8-bytes long value, or return an error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    ///
    /// # Example
    ///
    /// ```
    /// use buffer::*;
    ///
    /// let mut buf = ByteBuffer::from_bytes(&vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1]);
    /// let values = buf.read_u64().unwrap(); // value contains 1
    /// ```
    pub fn read_u64(&mut self) -> Result<u64> {
        read_number!(self, read_u64, 8)
    }

    /// Same as `read_u32()` method but for signed values
    pub fn read_i64(&mut self) -> Result<i64> {
        Ok(self.read_u64()? as i64)
    }

    /// Read a 32 bits floating point value, or return an error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn read_f32(&mut self) -> Result<f32> {
        read_number!(self, read_f32, 4)
    }

    /// Read a 64 bits floating point value, or return an error if not enough bytes are available.
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn read_f64(&mut self) -> Result<f64> {
        read_number!(self, read_f64, 8)
    }

    /// Read a string.
    /// _Note_: First, it reads a 32-bits values representing the string size,
    ///         then `size` raw bytes that must be encoded as UTF-8.
    ///
    /// _Note_: This method resets the read and write cursor for bitwise reading.
    pub fn read_string(&mut self) -> Result<String> {
        let size = self.read_u32()?;
        match String::from_utf8(self.read_bytes(size as usize)?) {
            Ok(s) => Ok(s),
            Err(e) => Err(anyhow!("invalid string data")),
        }
    }

    pub fn write_bit(&mut self, bit: Bit) {}

    pub fn write_bits(&mut self, value: u64, n: u8) {}

    pub fn read_bit(&mut self) -> Result<Bit> {
        todo!()
    }

    pub fn read_bits(&mut self, n: u8) -> Result<u64> {
        todo!()
    }

    pub fn flush_bit(&mut self) {
        todo!()
    }

    fn flush_w_bit(&mut self) {}

    fn flush_r_bit(&mut self) {}

    pub fn to_string(&self) -> String {
        todo!()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    pub fn get_r_pos(&self) -> usize {
        todo!()
    }

    pub fn set_r_pos(&mut self, r_pos: usize) {
        todo!()
    }

    pub fn get_w_pos(&self) -> usize {
        todo!()
    }

    pub fn set_w_pos(&self) {
        todo!()
    }
}
