use std::io::{Error, ErrorKind, Result};
use std::ops::RangeInclusive;
use std::str::FromStr;

use zerocopy::{FromBytes, IntoBytes};

use crate::symbols_and_data_types::{DataType, DataTypes, Symbol};

mod try_into;

#[derive(Clone, Debug, PartialEq)]
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
    Array(StartIndex, Vec<Variable>),
    Struct(Vec<(String, Variable)>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum StartIndex {
    Start,
    Some(i32),
}

impl Variable {
    pub(super) fn from_bytes(
        data_types: &DataTypes,
        symbol: &Symbol,
        symbol_data_type: &DataType,
        bytes: &[u8],
    ) -> Result<Self> {
        let array_ranges = symbol_data_type.array_ranges();
        Self::bytes_get_array(data_types, symbol, symbol_data_type, array_ranges, bytes)
    }

    fn bytes_get_array(
        data_types: &DataTypes,
        symbol: &Symbol,
        symbol_data_type: &DataType,
        array_ranges: &[RangeInclusive<i32>],
        bytes: &[u8],
    ) -> Result<Self> {
        let array_range = match array_ranges.first() {
            None => {
                return Self::bytes_get_variable_inner(data_types, symbol, symbol_data_type, bytes)
            }
            Some(ar) => ar,
        };

        let array_length = if array_range.end() < array_range.start() {
            return Ok(Self::Array(StartIndex::Some(*array_range.start()), vec![]));
        } else {
            (1 + array_range.end() - array_range.start()) as usize
        };
        let mut elements = Vec::new();
        let element_length = bytes.len() / array_length;
        if array_ranges.len() == 1 {
            let data_type = data_types.data_type_get_base_type(symbol_data_type)?;
            for index_actual in 0..array_length {
                elements.push(Self::bytes_get_variable_inner(
                    data_types,
                    symbol,
                    data_type,
                    &bytes[index_actual * element_length..(index_actual + 1) * element_length],
                )?);
            }
        } else {
            for index_actual in 0..array_length {
                elements.push(Self::bytes_get_array(
                    data_types,
                    symbol,
                    symbol_data_type,
                    &array_ranges[1..],
                    &bytes[index_actual * element_length..(index_actual + 1) * element_length],
                )?);
            }
        }
        Ok(Self::Array(
            StartIndex::Some(*array_range.start()),
            elements,
        ))
    }

    fn bytes_get_variable_inner(
        data_types: &DataTypes,
        symbol: &Symbol,
        symbol_data_type: &DataType,
        bytes: &[u8],
    ) -> Result<Self> {
        match symbol.data_type_id() {
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
            65 => Self::bytes_get_struct(data_types, symbol_data_type, bytes),
            31 => Err(Error::new(
                ErrorKind::Unsupported,
                format!(
                    "Type {} ({}) is not supported (value {bytes:?})",
                    symbol.data_type(),
                    symbol.data_type_id()
                ),
            )),
            32 | 34 => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Type {:?} ({}) is reserved (value {bytes:?})",
                    symbol.data_type(),
                    symbol.data_type_id()
                ),
            )),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Type {:?} ({}) is invalid (value {bytes:?})",
                    symbol.data_type(),
                    symbol.data_type_id()
                ),
            )),
        }
    }

    fn bytes_get_struct(
        data_types: &DataTypes,
        symbol_data_type: &DataType,
        bytes: &[u8],
    ) -> Result<Self> {
        let mut elements = Vec::new();
        for field in symbol_data_type.fields() {
            let field_data_type_name = field.data_type().trim();
            if field_data_type_name.contains("REFERENCE") {
                continue;
            }
            let field_data_type = data_types.get(field_data_type_name)?;
            let index_start = field.offset();
            let index_end = index_start + field_data_type.size_bytes();
            if index_end > bytes.len() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "{} has offset {} and size {} but byte length is {}",
                        field.name(),
                        field.offset(),
                        field_data_type.size_bytes(),
                        bytes.len()
                    ),
                ));
            }
            let field_bytes = &bytes[index_start..index_end];
            let field_value = Self::from_bytes(data_types, field, field_data_type, field_bytes)?;
            elements.push((field.name().to_string(), field_value));
        }
        Ok(Self::Struct(elements))
    }

    pub(super) fn to_bytes(&self, symbol: &Symbol, symbol_data_type: &DataType) -> Result<Vec<u8>> {
        self.to_bytes_inner(symbol, symbol_data_type.array_ranges())
    }

    pub(super) fn to_bytes_inner(
        &self,
        symbol: &Symbol,
        array_ranges: &[RangeInclusive<i32>],
    ) -> Result<Vec<u8>> {
        match (self, symbol.data_type_id()) {
            (Self::Void, 0) => Ok(Vec::new()),
            (Self::Bool(inner), 33) => {
                let byte: u8 = if *inner { 1 } else { 0 };
                Ok(vec![byte])
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
            (Self::Array(start_index, array_inner), _) => {
                array_to_bytes(symbol, start_index, array_inner, array_ranges)
            }
            (Self::Struct(_), 65) => Err(Error::new(
                ErrorKind::Unsupported,
                "Writing structs is not supported",
            )),
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Unexpected data type; expected {symbol:?}, got {self:?}"),
            )),
        }
    }
}

fn array_to_bytes(
    symbol: &Symbol,
    start_index: &StartIndex,
    array_content: &[Variable],
    array_ranges: &[RangeInclusive<i32>],
) -> Result<Vec<u8>> {
    match (array_ranges.first(), start_index) {
        (None, _) => Err(Error::new(
            ErrorKind::InvalidData,
            format!("Symbol Data Type is array but Data Type has no array range\n{symbol:?}"),
        )),
        (Some(array_range), StartIndex::Some(i)) if *array_range.start() != *i => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Array range is {array_range:?}, got start index {i}"),
        )),
        (Some(array_range), _)
            if array_content.len() as i32 > 1 + *array_range.end() - *array_range.start() =>
        {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Array range is {array_range:?}, got array of length {}",
                    array_content.len()
                ),
            ))
        }
        (Some(_), _) => {
            let mut output = Vec::new();
            if array_ranges.len() == 1 {
                for inner in array_content {
                    output.append(&mut inner.to_bytes_inner(symbol, &[])?);
                }
                Ok(output)
            } else {
                for inner_array in array_content {
                    if let Variable::Array(inner_start_index, inner) = inner_array {
                        output.append(&mut array_to_bytes(
                            symbol,
                            inner_start_index,
                            inner,
                            &array_ranges[1..],
                        )?);
                    } else {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            format!("Expected a nested array, got {inner_array:?}"),
                        ));
                    }
                }
                Ok(output)
            }
        }
    }
}

pub(super) fn str_and_symbol_to_bytes(value: &str, symbol: &Symbol) -> Result<Vec<u8>> {
    match symbol.data_type_id() {
        0 => {
            if value.is_empty() {
                Ok(vec![])
            } else {
                Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Expected void, got {value}"),
                ))
            }
        }
        33 => match bool::from_str(&value.to_lowercase()) {
            Ok(b) => {
                let byte = if b { 1 } else { 0 };
                Ok(vec![byte])
            }
            Err(e) => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected bool, got {value} ({e:?})"),
            )),
        },
        16 => from_str_to_bytes::<i8>(value),
        2 => from_str_to_bytes::<i16>(value),
        3 => from_str_to_bytes::<i32>(value),
        20 => from_str_to_bytes::<i64>(value),
        17 => from_str_to_bytes::<u8>(value),
        18 => from_str_to_bytes::<u16>(value),
        19 => from_str_to_bytes::<u32>(value),
        21 => from_str_to_bytes::<u64>(value),
        4 => from_str_to_bytes::<f32>(value),
        5 => from_str_to_bytes::<f64>(value),
        30 => Ok(str_to_bytes(value)),
        65 | 31 => Err(Error::new(
            ErrorKind::Unsupported,
            format!(
                "Type {} ({}) is not supported (value {value})",
                symbol.data_type(),
                symbol.data_type_id()
            ),
        )),
        32 | 34 => Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Type {:?} ({}) is reserved (value {value})",
                symbol.data_type(),
                symbol.data_type_id()
            ),
        )),
        _ => Err(Error::new(
            ErrorKind::InvalidData,
            format!("Unexpected data type\n{symbol:?}"),
        )),
    }
}

pub(super) fn bytes_to_inner<T: FromBytes>(bytes: &[u8]) -> Result<T> {
    match T::read_from_bytes(bytes) {
        Ok(t) => Ok(t),
        Err(e) => Err(Error::new(
            ErrorKind::InvalidData,
            format!("Cannot parse {bytes:?}\n{e:?}"),
        )),
    }
}

fn from_str_to_bytes<T: FromStr + zerocopy::Immutable + IntoBytes>(value: &str) -> Result<Vec<u8>> {
    match T::from_str(value) {
        Ok(t) => Ok(t.as_bytes().to_vec()),
        Err(_) => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Cannot parse {value}"),
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
