use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result};

use super::{beckhoff, result};

#[derive(Clone, Debug, Default)]
pub struct Symbols {
    symbols: HashMap<String, SymbolInfo>,
    data_types: HashMap<String, DataTypeInfo>,
}

#[derive(Clone, Debug)]
pub struct SymbolInfo {
    name: String,
    data_type: SymbolDataType,
    _comment: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SymbolDataType {
    id: u32,
    name: String,
}

#[derive(Clone, Debug)]
pub struct DataTypeInfo {
    name: String,
    size_bytes: u32,
    _comment: Option<String>,
    fields: Vec<SymbolInfo>,
}

impl Symbols {
    pub fn get_symbol_and_data_type(
        &self,
        value_name: &str,
    ) -> Result<(&SymbolInfo, &DataTypeInfo)> {
        let symbol_info = self.get_symbol(value_name)?;
        let data_type_info = if value_name.contains("[") && value_name.contains("]") {
            self.get_base_type_info(symbol_info)?
        } else {
            match self.data_types.get(&symbol_info.data_type.name) {
                Some(dti) => dti,
                None => {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        format!("Cannot find data type info for\n{symbol_info:?}"),
                    ))
                }
            }
        };

        Ok((symbol_info, data_type_info))
    }

    fn get_symbol(&self, value_name: &str) -> Result<&SymbolInfo> {
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
                let two_base = str_trim_array_accessor(two);
                format!("{one}.{two_base}")
            }
        };

        let mut symbol_entry = match self.symbols.get(&entry_name) {
            Some(en) => en,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Cannot find symbol entry for {value_name}"),
                ))
            }
        };

        for token in &tokens[2..] {
            let token_base = str_trim_array_accessor(token);
            let parent_data_type = self.get_base_type_info(symbol_entry)?;
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

    fn get_base_type_info(&self, symbol_info: &SymbolInfo) -> Result<&DataTypeInfo> {
        let base_type_as_string = match symbol_info.data_type.name.rfind(" OF ") {
            Some(start) => symbol_info.data_type.name[start + 4..].trim(),
            None => &symbol_info.data_type.name,
        };

        let base_type_as_string = match base_type_as_string.rfind(" TO ") {
            Some(start) => base_type_as_string[start + 4..].trim(),
            None => base_type_as_string,
        };

        match self.data_types.get(base_type_as_string) {
            Some(dt) => Ok(dt),
            None => Err(Error::new(
                ErrorKind::NotFound,
                format!("Cannot find data type for {symbol_info:?}"),
            )),
        }
    }
}

pub fn upload(ams_address: &beckhoff::AmsAddr, ads_port: i32) -> Result<Symbols> {
    let mut symbols = Symbols::default();

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
) -> Result<HashMap<String, SymbolInfo>> {
    let mut output = HashMap::new();

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
        let (symbol_info, n_bytes) = SymbolInfo::from_bytes(&buffer[offset..])?;
        output.insert(symbol_info.name.clone(), symbol_info);

        offset += n_bytes;
    }

    Ok(output)
}

fn upload_data_types(
    ams_address: &beckhoff::AmsAddr,
    ads_port: i32,
    size: u32,
    n: u32,
) -> Result<HashMap<String, DataTypeInfo>> {
    let mut output = HashMap::new();

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
        let (data_type_info, n_bytes) = DataTypeInfo::from_bytes(&buffer[offset..])?;
        output.insert(data_type_info.name.clone(), data_type_info);

        offset += n_bytes;
    }

    Ok(output)
}

impl SymbolInfo {
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

        let data_type = SymbolDataType {
            id: entry.dataType,
            name: data_type_name,
        };

        Ok((
            Self {
                name: bytes_get_string(&bytes[name_start..name_end])?,
                data_type,
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

        let data_type = SymbolDataType {
            id: entry.dataType,
            name: data_type_name,
        };

        Ok((
            Self {
                name: bytes_get_string(&bytes[name_start..name_end])?,
                data_type,
                _comment: comment,
            },
            entry.entryLength as usize,
        ))
    }

    pub(super) fn data_type(&self) -> &SymbolDataType {
        &self.data_type
    }
}

impl SymbolDataType {
    pub(super) fn id(&self) -> u32 {
        self.id
    }
}

impl DataTypeInfo {
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

        let comment = bytes_get_comment(&bytes[comment_start..comment_end])?;

        let mut fields = Vec::new();
        let mut field_this_start = field_info_start;
        for _ in 0..entry.subItems {
            let (field, length) = SymbolInfo::field_from_bytes(&bytes[field_this_start..])?;
            fields.push(field);
            field_this_start += length;
        }

        Ok((
            Self {
                name: bytes_get_string(&bytes[name_start..name_end])?,
                size_bytes: entry.size,
                _comment: comment,
                fields,
            },
            entry.entryLength as usize,
        ))
    }

    pub(super) fn size_bytes(&self) -> u32 {
        self.size_bytes
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

fn str_trim_array_accessor(input: &str) -> String {
    let mut output = input.to_string();
    if let Some(index) = output.find("[") {
        while output.len() > index {
            output.pop();
        }
    }
    output
}
