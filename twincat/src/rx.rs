use std::io::{Error, Result};

use super::client::Client;
use super::variables::Variable;
use super::{beckhoff, result};

impl Client {
    pub fn get_value(&self, value_name: impl AsRef<str>) -> Result<Variable> {
        let data_types = self.symbols_and_data_types().data_types();
        let (symbol_info, data_type_info) = self
            .symbols_and_data_types()
            .get_symbol_and_data_type(value_name.as_ref())?;
        let bytes = self.get_raw_bytes(value_name.as_ref(), data_type_info.size_bytes())?;
        Variable::from_bytes(data_types, symbol_info, data_type_info, &bytes)
    }

    fn get_raw_bytes(&self, value_name: &str, symbol_size_bytes: usize) -> Result<Vec<u8>> {
        const SIZE_SYMBOL_ENTRY: u32 = std::mem::size_of::<beckhoff::AdsSymbolEntry>() as u32;

        let ptr_address = &mut self.ams_address().to_owned() as *mut beckhoff::AmsAddr;

        let mut symbol_entry = beckhoff::AdsSymbolEntry::default();
        let ptr_symbol_entry =
            &mut symbol_entry as *mut beckhoff::AdsSymbolEntry as *mut std::os::raw::c_void;

        let ptr_name = value_name as *const str as *mut std::os::raw::c_void;

        result::process(unsafe {
            beckhoff::AdsSyncReadWriteReqEx2(
                self.port(),
                ptr_address,
                beckhoff::ADSIGRP_SYM_INFOBYNAMEEX,
                0,
                SIZE_SYMBOL_ENTRY,
                ptr_symbol_entry,
                value_name.len() as u32,
                ptr_name,
                std::ptr::null_mut(),
            )
        })?;

        let mut buffer = vec![0; symbol_size_bytes];
        let ptr_buffer = buffer.as_mut_ptr() as *mut std::os::raw::c_void;

        let mut n_bytes_read = 0_u32;
        let ptr_n_bytes_read = &mut n_bytes_read as *mut u32;

        result::process(unsafe {
            beckhoff::AdsSyncReadReqEx2(
                self.port(),
                ptr_address,
                symbol_entry.iGroup,
                symbol_entry.iOffs,
                symbol_size_bytes as u32,
                ptr_buffer,
                ptr_n_bytes_read,
            )
        })?;

        if cfg!(debug_assertions) && n_bytes_read as usize != symbol_size_bytes {
            return Err(Error::other(format!(
                "Expected to read {symbol_size_bytes} bytes, read {n_bytes_read} bytes"
            )));
        }

        Ok(buffer)
    }
}
