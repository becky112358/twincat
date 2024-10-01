use std::io::Result;

use super::{beckhoff, symbols};

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

        let symbols = match symbols::upload(&self.ams_address, port) {
            Ok(s) => s,
            Err(e) => {
                unsafe { beckhoff::AdsPortCloseEx(port) };
                unsafe { beckhoff::AdsPortClose() };
                return Err(e);
            }
        };

        Ok(Client {
            _ams_address: self.ams_address,
            port,
            symbols,
        })
    }
}

#[derive(Clone)]
pub struct Client {
    _ams_address: beckhoff::AmsAddr,
    port: i32,
    symbols: symbols::Symbols,
}

impl Drop for Client {
    fn drop(&mut self) {
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

    pub fn symbols(&self) -> &symbols::Symbols {
        &self.symbols
    }
}
