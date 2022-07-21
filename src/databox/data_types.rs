use crate::common::error::DBError;
use anyhow::Result;
use bytes::{Buf, Bytes};
use std::any::Any;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub enum DataType {
    Boolean,
    Integer,
    Float,
    String(usize),
    Long,
    ByteArray(usize),
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DataType::Boolean => "BOOLEAN",
            DataType::Integer => "INTEGER",
            DataType::Float => "FLOAT",
            DataType::String(_) => "STRING",
            DataType::Long => "LONG",
            DataType::ByteArray(_) => "BYTEARRAY",
        })
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum DataBox {
    Null,
    Boolean(bool),
    Integer(i32),
    Long(i64),
    Float(f64),
    String(String),
    ByteArray(Vec<u8>),
}

impl Eq for DataBox {}

impl Hash for DataBox {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.datatype().hash(state);
        match self {
            DataBox::Null => self.hash(state),
            DataBox::Boolean(v) => v.hash(state),
            DataBox::Integer(v) => v.hash(state),
            DataBox::Long(v) => v.hash(state),
            DataBox::Float(v) => v.to_be_bytes().hash(state),
            DataBox::String(v) => v.hash(state),
            DataBox::ByteArray(v) => v.hash(state),
        }
    }
}

impl Display for DataBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self {
                DataBox::Null => "NULL".to_string(),
                DataBox::Boolean(v) if *v => "TRUE".to_string(),
                DataBox::Boolean(_) => "FALSE".to_string(),
                DataBox::Integer(v) => v.to_string(),
                DataBox::Long(v) => v.to_string(),
                DataBox::Float(v) => v.to_string(),
                DataBox::String(v) => v.clone(),
                DataBox::ByteArray(v) => String::from_utf8(v.clone()).unwrap(),
            }
            .as_ref(),
        )
    }
}

impl DataBox {
    pub fn from_bytes(mut buf: Bytes, datatype: DataType) -> Result<Self> {
        match datatype {
            DataType::Boolean => Ok(DataBox::Boolean(buf.get_u8() == 1)),
            DataType::Integer => Ok(DataBox::Integer(buf.get_i32())),
            DataType::Float => Ok(DataBox::Float(buf.get_f64())),
            DataType::Long => Ok(DataBox::Long(buf.get_i64())),
            DataType::String(len) => {
                let mut dst: Vec<u8> = Vec::with_capacity(len);
                buf.copy_to_slice(dst.as_mut_slice());
                Ok(DataBox::String(String::from_utf8(dst)?))
            }
            DataType::ByteArray(len) => {
                let mut dst: Vec<u8> = Vec::with_capacity(len);
                buf.copy_to_slice(dst.as_mut_slice());
                Ok(DataBox::ByteArray(dst))
            }
        }
    }

    pub fn from_string(mut s: String, datatype: DataType) -> Result<Self> {
        todo!()
    }

    pub fn from_object(any: &dyn Any) -> Result<Self> {
        todo!()
    }

    pub fn datatype(&self) -> Option<DataType> {
        match self {
            Self::Null => None,
            Self::Boolean(_) => Some(DataType::Boolean),
            Self::Integer(_) => Some(DataType::Integer),
            Self::Long(_) => Some(DataType::Long),
            Self::Float(_) => Some(DataType::Float),
            Self::String(v) => Some(DataType::String(v.len())),
            Self::ByteArray(v) => Some(DataType::ByteArray(v.len())),
        }
    }

    pub fn boolean(self) -> Result<bool, DBError> {
        match self {
            Self::Boolean(b) => Ok(b),
            v => Err(DBError::TypeError(v, "boolean")),
        }
    }

    pub fn integer(self) -> Result<i32, DBError> {
        match self {
            DataBox::Integer(i) => Ok(i),
            v => Err(DBError::TypeError(v, "integer")),
        }
    }

    pub fn long(self) -> Result<i64, DBError> {
        match self {
            DataBox::Long(l) => Ok(l),
            v => Err(DBError::TypeError(v, "long")),
        }
    }

    pub fn float(self) -> Result<f64, DBError> {
        match self {
            DataBox::Float(f) => Ok(f),
            v => Err(DBError::TypeError(v, "float")),
        }
    }

    pub fn string(self) -> Result<String, DBError> {
        match self {
            DataBox::String(s) => Ok(s),
            v => Err(DBError::TypeError(v, "string")),
        }
    }

    pub fn byte_array(self) -> Result<Vec<u8>, DBError> {
        match self {
            DataBox::ByteArray(bytes) => Ok(bytes),
            v => Err(DBError::TypeError(v, "byte array")),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            DataBox::Null => vec![],
            DataBox::Boolean(v) => vec![*v as u8],
            DataBox::Integer(v) => v.to_be_bytes().to_vec(),
            DataBox::Long(v) => v.to_be_bytes().to_vec(),
            DataBox::Float(v) => v.to_be_bytes().to_vec(),
            DataBox::String(v) => v.clone().into_bytes(),
            DataBox::ByteArray(v) => v.to_vec(),
        }
    }

    pub fn hash_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl<'a> From<DataBox> for Cow<'a, DataBox> {
    fn from(v: DataBox) -> Self {
        Cow::Owned(v)
    }
}

impl<'a> From<&'a DataBox> for Cow<'a, DataBox> {
    fn from(v: &'a DataBox) -> Self {
        Cow::Borrowed(v)
    }
}

impl From<bool> for DataBox {
    fn from(v: bool) -> Self {
        DataBox::Boolean(v)
    }
}

impl From<i32> for DataBox {
    fn from(v: i32) -> Self {
        DataBox::Integer(v)
    }
}

impl From<i64> for DataBox {
    fn from(v: i64) -> Self {
        DataBox::Long(v)
    }
}

impl From<f64> for DataBox {
    fn from(v: f64) -> Self {
        DataBox::Float(v)
    }
}

impl From<String> for DataBox {
    fn from(v: String) -> Self {
        DataBox::String(v)
    }
}

impl From<&str> for DataBox {
    fn from(v: &str) -> Self {
        DataBox::String(v.to_owned())
    }
}

impl From<Vec<u8>> for DataBox {
    fn from(v: Vec<u8>) -> Self {
        DataBox::ByteArray(v)
    }
}

impl From<&Vec<u8>> for DataBox {
    fn from(v: &Vec<u8>) -> Self {
        DataBox::ByteArray(v.to_owned())
    }
}

impl From<&[u8]> for DataBox {
    fn from(v: &[u8]) -> Self {
        DataBox::ByteArray(v.to_vec())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_bool_type() {}
}
