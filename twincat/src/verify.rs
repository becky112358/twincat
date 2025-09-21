use std::io::Result;

use super::variables::{self, Variable};
use super::Client;

impl Client {
    /// A function for verifying an ADS path without actually invoking an ADS Client call
    pub fn verify_ads_path(&self, value_name: impl AsRef<str>) -> Result<()> {
        let _ = self
            .symbols_and_data_types()
            .get_symbol_and_data_type(value_name.as_ref())?;
        Ok(())
    }

    /// A function for verifying an ADS path and associated Variable type
    /// without actually invoking an ADS Client call
    pub fn verify_ads_path_and_variable_type(
        &self,
        value_name: impl AsRef<str>,
        value: Variable,
    ) -> Result<()> {
        let data_types = self.symbols_and_data_types().data_types();
        let (symbol_info, data_type_info) = self
            .symbols_and_data_types()
            .get_symbol_and_data_type(value_name.as_ref())?;
        let _ = value.to_bytes(data_types, symbol_info, data_type_info)?;
        Ok(())
    }

    /// A function for verifying an ADS path and associated Variable
    /// without actually invoking an ADS Client call
    pub fn verify_ads_path_and_str_variable(
        &self,
        value_name: impl AsRef<str>,
        value: &str,
    ) -> Result<()> {
        let (symbol_info, data_type_info) = self
            .symbols_and_data_types()
            .get_symbol_and_data_type(value_name.as_ref())?;
        let _ = variables::str_and_symbol_to_bytes(value, symbol_info, data_type_info)?;
        Ok(())
    }
}
