use std::io::Result;

use super::{beckhoff, symbols_and_data_types};

pub struct ClientBuilder {
    ams_address: beckhoff::AmsAddr,
}

impl ClientBuilder {
    pub fn with_ams_address(mut self, address: [u8; 6]) -> Self {
        self.ams_address.netId.b = address;
        self
    }

    pub fn with_ams_port(mut self, port: u16) -> Self {
        self.ams_address.port = port;
        self
    }

    pub fn connect(&self) -> Result<Client> {
        unsafe { beckhoff::AdsPortOpen() };
        let port = unsafe { beckhoff::AdsPortOpenEx() };

        let symbols_and_data_types = match symbols_and_data_types::upload(&self.ams_address, port) {
            Ok(s) => s,
            Err(e) => {
                unsafe { beckhoff::AdsPortCloseEx(port) };
                unsafe { beckhoff::AdsPortClose() };
                return Err(e);
            }
        };

        Ok(Client {
            ams_address: self.ams_address,
            port,
            symbols_and_data_types,
        })
    }
}

#[derive(Clone)]
pub struct Client {
    ams_address: beckhoff::AmsAddr,
    port: i32,
    symbols_and_data_types: symbols_and_data_types::SymbolsAndDataTypes,
}

impl Drop for Client {
    fn drop(&mut self) {
        #[cfg(feature = "notifications")]
        let _ = self.drop_notification_requests();

        unsafe { beckhoff::AdsPortCloseEx(self.port) };
        unsafe { beckhoff::AdsPortClose() };
    }
}

impl Client {
    pub fn builder() -> ClientBuilder {
        let mut ams_address = beckhoff::AmsAddr::default();
        unsafe { beckhoff::AdsGetLocalAddress(&mut ams_address) };

        ClientBuilder { ams_address }
    }

    pub(super) fn ams_address(&self) -> &beckhoff::AmsAddr {
        &self.ams_address
    }
    pub(super) fn port(&self) -> i32 {
        self.port
    }
    pub fn symbols_and_data_types(&self) -> &symbols_and_data_types::SymbolsAndDataTypes {
        &self.symbols_and_data_types
    }
}
