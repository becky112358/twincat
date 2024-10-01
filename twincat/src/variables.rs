use std::io::{Error, ErrorKind, Result};

use zerocopy::FromBytes;

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
