use std::io::Result;

use super::client::Client;
use super::variables::Variable;
use super::{beckhoff, result};

impl Client {
    pub fn get_value(&self, value_name: impl AsRef<str>) -> Result<Variable> {
        let (symbol_info, data_type_info) = self
            .symbols()
            .get_symbol_and_data_type(value_name.as_ref())?;
        let bytes = self.get_raw_bytes(value_name.as_ref(), data_type_info.size_bytes())?;
        Variable::from_bytes(symbol_info, data_type_info, &bytes)
    }

    fn get_raw_bytes(&self, value_name: &str, symbol_size_bytes: u32) -> Result<Vec<u8>> {
        const SIZE_U32: u32 = std::mem::size_of::<u32>() as u32;

        let ptr_address = &mut self.ams_address().to_owned() as *mut beckhoff::AmsAddr;

        let mut handle = 0;
        let ptr_handle = &mut handle as *mut u32 as *mut std::os::raw::c_void;

        let ptr_name = value_name as *const str as *mut std::os::raw::c_void;

        result::process(unsafe {
            beckhoff::AdsSyncReadWriteReqEx2(
                self.port(),
                ptr_address,
                beckhoff::ADSIGRP_SYM_HNDBYNAME,
                0,
                SIZE_U32,
                ptr_handle,
                value_name.len() as u32,
                ptr_name,
                std::ptr::null_mut(),
            )
        })?;

        let mut buffer = vec![0; symbol_size_bytes as usize];
        let ptr_buffer = buffer.as_mut_ptr() as *mut std::os::raw::c_void;

        result::process(unsafe {
            beckhoff::AdsSyncReadReqEx2(
                self.port(),
                ptr_address,
                beckhoff::ADSIGRP_SYM_VALBYHND,
                handle,
                symbol_size_bytes,
                ptr_buffer,
                std::ptr::null_mut(),
            )
        })?;

        result::process(unsafe {
            beckhoff::AdsSyncWriteReqEx(
                self.port(),
                ptr_address,
                beckhoff::ADSIGRP_SYM_RELEASEHND,
                0,
                SIZE_U32,
                ptr_handle,
            )
        })?;

        Ok(buffer)
    }
}
