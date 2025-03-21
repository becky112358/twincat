use std::io::Result;

use super::client::Client;
use super::variables::{self, Variable};
use super::{beckhoff, result};

impl Client {
    pub fn set_value(&self, value_name: impl AsRef<str>, value: Variable) -> Result<()> {
        let (symbol_info, _) = self
            .symbols_and_data_types()
            .get_symbol_and_data_type(value_name.as_ref())?;
        let bytes = value.to_bytes(symbol_info)?;
        self.set_raw_bytes(value_name.as_ref(), bytes)?;
        Ok(())
    }

    pub fn set_value_from_str(&self, value_name: impl AsRef<str>, value: &str) -> Result<()> {
        let (symbol_info, _) = self
            .symbols_and_data_types()
            .get_symbol_and_data_type(value_name.as_ref())?;
        let bytes = variables::str_and_symbol_to_bytes(value, symbol_info)?;
        self.set_raw_bytes(value_name.as_ref(), bytes)?;
        Ok(())
    }

    fn set_raw_bytes(&self, value_name: &str, bytes: Vec<u8>) -> Result<()> {
        const SIZE_SYMBOL_ENTRY: u32 = std::mem::size_of::<beckhoff::AdsSymbolEntry>() as u32;

        let ptr_address = &mut self.ams_address().to_owned() as *mut beckhoff::AmsAddr;

        let mut symbol_entry = beckhoff::AdsSymbolEntry::default();
        let ptr_symbol_entry =
            &mut symbol_entry as *mut beckhoff::AdsSymbolEntry as *mut std::os::raw::c_void;

        let mut bytes = bytes;
        let ptr_bytes = bytes.as_mut_ptr() as *mut std::os::raw::c_void;

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

        result::process(unsafe {
            beckhoff::AdsSyncWriteReqEx(
                self.port(),
                ptr_address,
                symbol_entry.iGroup,
                symbol_entry.iOffs,
                bytes.len() as u32,
                ptr_bytes,
            )
        })?;

        Ok(())
    }
}
