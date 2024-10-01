use std::io::{Error, ErrorKind, Result};

use zerocopy::{AsBytes, FromBytes};

use super::symbols::SymbolInfo;

#[derive(Debug, PartialEq)]
pub enum Variable {
    Void,
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
}

impl Variable {
    pub(super) fn from_bytes(symbol_info: &SymbolInfo, bytes: &[u8]) -> Result<Self> {
        match symbol_info.data_type().id() {
            0 => {
                if bytes.is_empty() {
                    Ok(Self::Void)
                } else {
                    Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Type void has value {bytes:?}"),
                    ))
                }
            }
            33 => {
                let byte: u8 = bytes_to_inner(bytes)?;
                if byte == 0 {
                    Ok(Self::Bool(false))
                } else if byte == 1 {
                    Ok(Self::Bool(true))
                } else {
                    Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Type bool has value {bytes:?}"),
                    ))
                }
            }
            16 => Ok(Self::I8(bytes_to_inner(bytes)?)),
            2 => Ok(Self::I16(bytes_to_inner(bytes)?)),
            3 => Ok(Self::I32(bytes_to_inner(bytes)?)),
            20 => Ok(Self::I64(bytes_to_inner(bytes)?)),
            17 => Ok(Self::U8(bytes_to_inner(bytes)?)),
            18 => Ok(Self::U16(bytes_to_inner(bytes)?)),
            19 => Ok(Self::U32(bytes_to_inner(bytes)?)),
            21 => Ok(Self::U64(bytes_to_inner(bytes)?)),
            4 => Ok(Self::F32(bytes_to_inner(bytes)?)),
            5 => Ok(Self::F64(bytes_to_inner(bytes)?)),
            30 => Ok(Self::String(bytes_to_string(bytes)?)),
            31 | 65 => Err(Error::new(
                ErrorKind::Unsupported,
                format!(
                    "Type {:?} is not (yet?) supported (value {bytes:?})",
                    symbol_info.data_type()
                ),
            )),
            32 | 34 => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Type {:?} is reserved (value {bytes:?})",
                    symbol_info.data_type()
                ),
            )),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Type {:?} is invalid (value {bytes:?})",
                    symbol_info.data_type()
                ),
            )),
        }
    }

    pub(super) fn to_bytes(&self, symbol_info: &SymbolInfo) -> Result<Vec<u8>> {
        match (self, symbol_info.data_type().id()) {
            (Self::Void, 0) => Ok(Vec::new()),
            (Self::Bool(inner), 33) => {
                let byte: u8 = if *inner { 1 } else { 0 };
                Ok(byte.as_bytes().to_vec())
            }
            (Self::I8(inner), 16) => Ok(inner.as_bytes().to_vec()),
            (Self::I16(inner), 2) => Ok(inner.as_bytes().to_vec()),
            (Self::I32(inner), 3) => Ok(inner.as_bytes().to_vec()),
            (Self::I64(inner), 20) => Ok(inner.as_bytes().to_vec()),
            (Self::U8(inner), 17) => Ok(inner.as_bytes().to_vec()),
            (Self::U16(inner), 18) => Ok(inner.as_bytes().to_vec()),
            (Self::U32(inner), 19) => Ok(inner.as_bytes().to_vec()),
            (Self::U64(inner), 21) => Ok(inner.as_bytes().to_vec()),
            (Self::F32(inner), 4) => Ok(inner.as_bytes().to_vec()),
            (Self::F64(inner), 5) => Ok(inner.as_bytes().to_vec()),
            (Self::String(inner), 30) => Ok(str_to_bytes(inner)),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Unexpected data type; expected {symbol_info:?}, got {self:?}"),
            )),
        }
    }
}

pub(super) fn bytes_to_inner<T: FromBytes>(bytes: &[u8]) -> Result<T> {
    match T::read_from(bytes) {
        Some(t) => Ok(t),
        None => Err(Error::new(
            ErrorKind::InvalidData,
            format!("Cannot parse {bytes:?}"),
        )),
    }
}

fn bytes_to_string(bytes: &[u8]) -> Result<String> {
    let end_index = match bytes.iter().position(|&c| c == 0) {
        Some(ei) => ei,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Cannot find null-terminator\n{bytes:?}"),
            ))
        }
    };

    match String::from_utf8(bytes[..end_index].to_vec()) {
        Ok(s) => Ok(s),
        Err(err) => Err(Error::new(ErrorKind::Other, err.to_string())),
    }
}

fn str_to_bytes(inner: &str) -> Vec<u8> {
    let mut output = inner.as_bytes().to_vec();
    output.push(0);
    output
}
