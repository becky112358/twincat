use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};
use std::ops::RangeInclusive;

use super::{beckhoff, result};

mod array;
mod filters;

#[derive(Clone, Debug, Default)]
pub struct SymbolsAndDataTypes {
    symbols: Symbols,
    data_types: DataTypes,
}

#[derive(Clone, Debug, Default)]
pub struct Symbols(HashMap<String, Symbol>);

#[derive(Clone, Debug, Default)]
pub struct DataTypes(HashMap<String, DataType>);

#[derive(Clone, Debug)]
pub struct Symbol {
    name: String,
    data_type_id: u8,
    data_type_name: String,
    offset: usize,
    group: Group,
    persistent: bool,
    _comment: Option<String>,
}

#[derive(Clone, Debug)]
enum Group {
    Input,  // %I*
    Output, // %Q*
    Flag,   // %M*
    None,
    StructField,
}

#[derive(Clone, Debug)]
pub struct DataType {
    name: String,
    size_bytes: usize,
    _comment: Option<String>,
    array_ranges: Vec<RangeInclusive<i32>>,
    fields: Vec<Symbol>,
}

impl SymbolsAndDataTypes {
    pub(super) fn get_symbol_and_data_type(
        &self,
        value_name: &str,
    ) -> Result<(&Symbol, &DataType)> {
        let symbol = self.get_symbol(value_name)?;

        let n_array_accessings = array::count_accessors(value_name);
        let data_type_info = if n_array_accessings > 0 {
            self.data_types
                .symbol_get_base_type(symbol, Some(n_array_accessings))?
        } else {
            match self.data_types.0.get(&symbol.data_type_name) {
                Some(dti) => dti,
                None => {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        format!("Cannot find data type info for\n{symbol:?}"),
                    ))
                }
            }
        };

        Ok((symbol, data_type_info))
    }

    fn get_symbol(&self, value_name: &str) -> Result<&Symbol> {
        let tokens = value_name.split('.').collect::<Vec<&str>>();
        let entry_name = match tokens[..] {
            [] => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Cannot find {value_name} (it seems to be empty)"),
                ))
            }
            [one] => one.to_string(),
            [one, two, ..] => {
                let two_base = array::trim_accessors(two);
                format!("{one}.{two_base}")
            }
        };

        let mut symbol_entry = match self.symbols.0.get(&entry_name) {
            Some(en) => en,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Cannot find symbol entry for {value_name}"),
                ))
            }
        };

        for token in &tokens[2..] {
            let token_base = array::trim_accessors(token);
            let parent_data_type = self.data_types.symbol_get_base_type(symbol_entry, None)?;
            let mut found = false;
            for field in &parent_data_type.fields {
                if field.name == *token_base {
                    symbol_entry = field;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Cannot find {token} in {parent_data_type:?}"),
                ));
            }
        }

        Ok(symbol_entry)
    }

    pub fn symbols(&self) -> &Symbols {
        &self.symbols
    }
    pub(super) fn data_types(&self) -> &DataTypes {
        &self.data_types
    }
}

impl DataTypes {
    pub(super) fn get(&self, name: &str) -> Result<&DataType> {
        match self.0.get(name) {
            Some(dt) => Ok(dt),
            None => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Cannot find {name} in data types"),
            )),
        }
    }

    pub(super) fn data_type_get_base_type(&self, data_type: &DataType) -> Result<&DataType> {
        let base_type = data_type_get_base_name(&data_type.name, None)?;
        self.get(base_type)
    }

    fn symbol_get_base_type(
        &self,
        symbol: &Symbol,
        n_array_accessings: Option<u8>,
    ) -> Result<&DataType> {
        let base_type = data_type_get_base_name(&symbol.data_type_name, n_array_accessings)?;
        self.get(base_type)
    }
}

fn data_type_get_base_name(data_type: &str, n_array_accessings: Option<u8>) -> Result<&str> {
    let base_name = array::get_base_name(data_type, n_array_accessings)?;

    let base_name = match base_name.rfind(" TO ") {
        Some(start) => base_name[start + 4..].trim(),
        None => base_name,
    };

    Ok(base_name)
}

pub(super) fn upload(
    ams_address: &beckhoff::AmsAddr,
    ads_port: i32,
) -> Result<SymbolsAndDataTypes> {
    let mut symbols = SymbolsAndDataTypes::default();

    let mut address = *ams_address;
    let ptr_address = &mut address as *mut beckhoff::AmsAddr;

    let mut upload_info = beckhoff::AdsSymbolUploadInfo2::default();
    let ptr_upload_info =
        &mut upload_info as *mut beckhoff::AdsSymbolUploadInfo2 as *mut std::os::raw::c_void;

    result::process(unsafe {
        beckhoff::AdsSyncReadReqEx2(
            ads_port,
            ptr_address,
            beckhoff::ADSIGRP_SYM_UPLOADINFO2,
            0,
            std::mem::size_of::<beckhoff::AdsSymbolUploadInfo2>() as u32,
            ptr_upload_info,
            std::ptr::null_mut(),
        )
    })?;

    symbols.symbols = upload_symbols(
        ams_address,
        ads_port,
        upload_info.nSymSize,
        upload_info.nSymbols,
    )?;

    symbols.data_types = upload_data_types(
        ams_address,
        ads_port,
        upload_info.nDatatypeSize,
        upload_info.nDatatypes,
    )?;

    Ok(symbols)
}

fn upload_symbols(
    ams_address: &beckhoff::AmsAddr,
    ads_port: i32,
    size: u32,
    n: u32,
) -> Result<Symbols> {
    let mut output = Symbols(HashMap::new());

    let mut address = *ams_address;
    let ptr_address = &mut address as *mut beckhoff::AmsAddr;

    let mut buffer: Box<[u8]> = vec![0; size as usize].into_boxed_slice();
    let ptr_buffer = buffer.as_mut_ptr() as *mut std::os::raw::c_void;

    result::process(unsafe {
        beckhoff::AdsSyncReadReqEx2(
            ads_port,
            ptr_address,
            beckhoff::ADSIGRP_SYM_UPLOAD,
            0,
            size,
            ptr_buffer,
            std::ptr::null_mut(),
        )
    })?;

    let mut offset = 0;
    for _ in 0..n {
        let (symbol, n_bytes) = Symbol::from_bytes(&buffer[offset..])?;
        output.0.insert(symbol.name.clone(), symbol);

        offset += n_bytes;
    }

    Ok(output)
}

fn upload_data_types(
    ams_address: &beckhoff::AmsAddr,
    ads_port: i32,
    size: u32,
    n: u32,
) -> Result<DataTypes> {
    let mut output = DataTypes(HashMap::new());

    let mut address = *ams_address;
    let ptr_address = &mut address as *mut beckhoff::AmsAddr;

    let mut buffer: Box<[u8]> = vec![0; size as usize].into_boxed_slice();
    let ptr_buffer = buffer.as_mut_ptr() as *mut std::os::raw::c_void;

    result::process(unsafe {
        beckhoff::AdsSyncReadReqEx2(
            ads_port,
            ptr_address,
            beckhoff::ADSIGRP_SYM_DT_UPLOAD,
            0,
            size,
            ptr_buffer,
            std::ptr::null_mut(),
        )
    })?;

    let mut offset = 0;
    for _ in 0..n {
        let (data_type_info, n_bytes) = DataType::from_bytes(&buffer[offset..])?;
        output.0.insert(data_type_info.name.clone(), data_type_info);

        offset += n_bytes;
    }

    Ok(output)
}

impl Symbol {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize)> {
        const DETAILS_LENGTH: usize = std::mem::size_of::<beckhoff::AdsSymbolEntry>();

        let entry: &beckhoff::AdsSymbolEntry =
            unsafe { &*bytes[0..DETAILS_LENGTH].as_ptr().cast() };

        let name_start = DETAILS_LENGTH;
        let name_end = name_start + entry.nameLength as usize;
        let data_type_name_start = name_end + 1;
        let data_type_name_end = data_type_name_start + entry.typeLength as usize;
        let comment_start = data_type_name_end + 1;
        let comment_end = comment_start + entry.commentLength as usize;

        let data_type_name = bytes_get_string(&bytes[data_type_name_start..data_type_name_end])?;
        let comment = bytes_get_comment(&bytes[comment_start..comment_end])?;

        Ok((
            Self {
                name: bytes_get_string(&bytes[name_start..name_end])?,
                data_type_id: entry.dataType as u8,
                data_type_name,
                group: Group::from_u32(entry.iGroup),
                offset: entry.iOffs as usize,
                persistent: entry.flags & beckhoff::ADSSYMBOLFLAG_PERSISTENT
                    == beckhoff::ADSSYMBOLFLAG_PERSISTENT,
                _comment: comment,
            },
            entry.entryLength as usize,
        ))
    }

    fn field_from_bytes(bytes: &[u8]) -> Result<(Self, usize)> {
        const DETAILS_LENGTH: usize = std::mem::size_of::<beckhoff::AdsDatatypeEntry>();

        let entry: &beckhoff::AdsDatatypeEntry =
            unsafe { &*bytes[0..DETAILS_LENGTH].as_ptr().cast() };

        let name_start = DETAILS_LENGTH;
        let name_end = name_start + entry.nameLength as usize;
        let data_type_name_start = name_end + 1;
        let data_type_name_end = data_type_name_start + entry.typeLength as usize;
        let comment_start = data_type_name_end + 1;
        let comment_end = comment_start + entry.commentLength as usize;

        let data_type_name = bytes_get_string(&bytes[data_type_name_start..data_type_name_end])?;
        let comment = bytes_get_comment(&bytes[comment_start..comment_end])?;

        Ok((
            Self {
                name: bytes_get_string(&bytes[name_start..name_end])?,
                data_type_id: entry.dataType as u8,
                data_type_name,
                offset: entry.offs as usize,
                group: Group::StructField,
                persistent: (entry.flags >> 8) & beckhoff::ADSSYMBOLFLAG_PERSISTENT
                    == beckhoff::ADSSYMBOLFLAG_PERSISTENT,
                _comment: comment,
            },
            entry.entryLength as usize,
        ))
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }
    pub(super) fn data_type_id(&self) -> u8 {
        self.data_type_id
    }
    pub(super) fn data_type(&self) -> &str {
        &self.data_type_name
    }
    pub(super) fn offset(&self) -> usize {
        self.offset
    }
}

impl Group {
    fn from_u32(input: u32) -> Self {
        match input {
            61472 => Self::Input,
            61488 => Self::Output,
            16416 => Self::Flag,
            16448 => Self::None,
            _ => {
                if cfg!(debug_assertions) {
                    println!("Unrecognised group {input}");
                }
                Self::None
            }
        }
    }
}

impl DataType {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, usize)> {
        const DETAILS_LENGTH: usize = std::mem::size_of::<beckhoff::AdsDatatypeEntry>();
        const ARRAY_INFO_LENGTH: usize = std::mem::size_of::<beckhoff::AdsDatatypeArrayInfo>();

        let entry: &beckhoff::AdsDatatypeEntry =
            unsafe { &*bytes[0..DETAILS_LENGTH].as_ptr().cast() };

        let name_start = DETAILS_LENGTH;
        let name_end = name_start + entry.nameLength as usize;
        let type_start = name_end + 1;
        let type_end = type_start + entry.typeLength as usize;
        let comment_start = type_end + 1;
        let comment_end = comment_start + entry.commentLength as usize;
        let field_info_start = comment_end + 1 + (entry.arrayDim as usize * ARRAY_INFO_LENGTH);

        let name = bytes_get_string(&bytes[name_start..name_end])?;

        let comment = bytes_get_comment(&bytes[comment_start..comment_end])?;

        let array_ranges = array::get_ranges(&name).unwrap_or_default();

        let mut fields = Vec::new();
        let mut field_this_start = field_info_start;
        for _ in 0..entry.subItems {
            let (field, length) = Symbol::field_from_bytes(&bytes[field_this_start..])?;
            fields.push(field);
            field_this_start += length;
        }

        Ok((
            Self {
                name,
                size_bytes: entry.size as usize,
                _comment: comment,
                array_ranges,
                fields,
            },
            entry.entryLength as usize,
        ))
    }

    pub(super) fn size_bytes(&self) -> usize {
        self.size_bytes
    }
    pub(super) fn array_ranges(&self) -> &[RangeInclusive<i32>] {
        &self.array_ranges
    }
    pub(super) fn fields(&self) -> &[Symbol] {
        &self.fields
    }
}

fn bytes_get_comment(bytes: &[u8]) -> Result<Option<String>> {
    let comment_string = bytes_get_string(bytes)?;
    if comment_string.is_empty() {
        Ok(None)
    } else {
        Ok(Some(comment_string))
    }
}

fn bytes_get_string(bytes: &[u8]) -> Result<String> {
    if let Ok(s) = std::str::from_utf8(bytes) {
        Ok(s.trim().to_string())
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Cannot parse {bytes:?}"),
        ))
    }
}
