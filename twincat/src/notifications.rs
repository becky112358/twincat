use std::io::{Error, ErrorKind, Result};
use std::sync::RwLock;

use lazy_static::lazy_static;

use super::symbols_and_data_types::SymbolsAndDataTypes;
use super::{beckhoff, result};
use super::{Client, Variable};

pub enum AdsTransmissionMode {
    None,
    ClientCycle,
    ClientOnChange,
    Cyclic,
    OnChange,
}

pub enum Time {
    Seconds(u8),
    /// Number of milliseconds must not exceed 429496
    MilliSeconds(u32),
    /// Number of microseconds must not exceed 429496729
    MicroSeconds(u32),
    /// Number of nanoseconds must be a multiple of 100
    NanoSeconds(u32),
}

lazy_static! {
    static ref REGISTERED_SYMBOLS: RwLock<(Option<SymbolsAndDataTypes>, Vec<RegisteredSymbols>)> =
        RwLock::new((None, Vec::new()));
}

struct RegisteredSymbols {
    name: String,
    value_handle: u32,
    notification_handle: u32,
    callback: fn(&str, Variable),
}

impl Client {
    /// Multiple notifications can be requested for one value.
    /// This function returns the notification handle, which can be used to
    /// delete that specific notification request.
    pub fn request_notifications(
        &self,
        value_name: String,
        ads_transmission_mode: AdsTransmissionMode,
        max_delay: Option<Time>,
        cycle_time: Option<Time>,
        callback: fn(&str, Variable),
    ) -> Result<u32> {
        let ptr_address = &mut self.ams_address().to_owned() as *mut beckhoff::AmsAddr;

        let mut value_handle = 0_u32;
        let ptr_value_handle = &mut value_handle as *mut u32 as *mut std::os::raw::c_void;

        let ptr_name = value_name.as_ref() as *const str as *mut std::os::raw::c_void;

        result::process(unsafe {
            beckhoff::AdsSyncReadWriteReqEx2(
                self.port(),
                ptr_address,
                beckhoff::ADSIGRP_SYM_HNDBYNAME,
                0,
                std::mem::size_of::<u32>() as u32,
                ptr_value_handle,
                value_name.len() as u32,
                ptr_name,
                std::ptr::null_mut(),
            )
        })?;

        let (_, value_data_type) = self
            .symbols_and_data_types()
            .get_symbol_and_data_type(value_name.as_ref())?;

        let mut ads_notification_attribute = beckhoff::AdsNotificationAttrib {
            cbLength: value_data_type.size_bytes() as u32,
            nTransMode: ads_transmission_mode.to_beckhoff(),
            nMaxDelay: time_to_beckhoff(&max_delay)?,
            __bindgen_anon_1: beckhoff::AdsNotificationAttrib__bindgen_ty_1 {
                nCycleTime: time_to_beckhoff(&cycle_time)?,
            },
        };
        let ptr_ads_notification_attribute =
            &mut ads_notification_attribute as *mut beckhoff::AdsNotificationAttrib;

        let mut notification_handle = 0_u32;
        let ptr_notification_handle = &mut notification_handle as *mut u32;

        result::process(unsafe {
            beckhoff::AdsSyncAddDeviceNotificationReqEx(
                self.port(),
                ptr_address,
                beckhoff::ADSIGRP_SYM_VALBYHND,
                value_handle,
                ptr_ads_notification_attribute,
                Some(callback_wrapper),
                value_handle,
                ptr_notification_handle,
            )
        })?;

        let mut registered_symbols = match REGISTERED_SYMBOLS.write() {
            Ok(w) => w,
            Err(e) => return Err(Error::other(format!("Write-lock failure!\n{e}"))),
        };
        registered_symbols.1.push(RegisteredSymbols {
            name: value_name,
            value_handle,
            notification_handle,
            callback,
        });
        if registered_symbols.0.is_none() {
            registered_symbols.0 = Some(self.symbols_and_data_types().clone());
        }

        Ok(notification_handle)
    }

    /// Deletes all notification requests for this value
    pub fn delete_notifications_for_value(&self, value_name: impl AsRef<str>) -> Result<()> {
        let mut registered_symbols = match REGISTERED_SYMBOLS.write() {
            Ok(rs) => rs,
            Err(e) => return Err(Error::other(format!("Write-lock failure!\n{e}"))),
        };
        let mut found = false;
        for i in (0..registered_symbols.1.len()).rev() {
            if registered_symbols.1[i].name.eq(value_name.as_ref()) {
                self.delete_notification_request_from_handles(
                    registered_symbols.1[i].value_handle,
                    registered_symbols.1[i].notification_handle,
                )?;
                registered_symbols.1.remove(i);
                found = true;
            }
        }

        if !found {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Cannot find Notification for {}", value_name.as_ref()),
            ));
        }

        Ok(())
    }

    pub fn delete_notification_with_handle(&self, notification_handle: u32) -> Result<()> {
        let mut registered_symbols = match REGISTERED_SYMBOLS.write() {
            Ok(rs) => rs,
            Err(e) => return Err(Error::other(format!("Write-lock failure!\n{e}"))),
        };
        let mut details = None;
        for i in (0..registered_symbols.1.len()).rev() {
            if registered_symbols.1[i].notification_handle == notification_handle {
                details = Some(registered_symbols.1.remove(i));
                break;
            }
        }
        drop(registered_symbols);
        let details = match details {
            Some(d) => d,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Cannot find Notification Request with handle {notification_handle}",),
                ))
            }
        };

        self.delete_notification_request_from_handles(
            details.value_handle,
            details.notification_handle,
        )?;

        Ok(())
    }

    pub(super) fn drop_notification_requests(&self) -> Result<()> {
        let mut registered_symbols = match REGISTERED_SYMBOLS.write() {
            Ok(rs) => rs,
            Err(e) => return Err(Error::other(format!("Write-lock failure!\n{e}"))),
        };
        registered_symbols.0 = None;
        for i in (0..registered_symbols.1.len()).rev() {
            self.delete_notification_request_from_handles(
                registered_symbols.1[i].value_handle,
                registered_symbols.1[i].notification_handle,
            )?;
            registered_symbols.1.remove(i);
        }
        Ok(())
    }

    fn delete_notification_request_from_handles(
        &self,
        value_handle: u32,
        notification_handle: u32,
    ) -> Result<()> {
        let ptr_address = &mut self.ams_address().to_owned() as *mut beckhoff::AmsAddr;

        result::process(unsafe {
            beckhoff::AdsSyncDelDeviceNotificationReqEx(
                self.port(),
                ptr_address,
                notification_handle,
            )
        })?;

        let mut value_handle = value_handle;
        let ptr_value_handle = &mut value_handle as *mut u32 as *mut std::os::raw::c_void;

        result::process(unsafe {
            beckhoff::AdsSyncWriteReqEx(
                self.port(),
                ptr_address,
                beckhoff::ADSIGRP_SYM_RELEASEHND,
                0,
                std::mem::size_of::<u32>() as u32,
                ptr_value_handle,
            )
        })?;

        Ok(())
    }
}

unsafe extern "C" fn callback_wrapper(
    _: *mut beckhoff::AmsAddr,
    ptr_notification: *mut beckhoff::AdsNotificationHeader,
    value_handle: std::os::raw::c_ulong,
) {
    let registered_symbols = match REGISTERED_SYMBOLS.read() {
        Ok(rs) => rs,
        Err(_) => return,
    };
    let symbols_and_data_types = match &registered_symbols.0 {
        Some(sdt) => sdt,
        None => return,
    };
    for registered_symbol in registered_symbols.1.iter() {
        if registered_symbol.value_handle == value_handle {
            let sample_size = (*ptr_notification).cbSampleSize as usize;
            let data_slice =
                std::slice::from_raw_parts((*ptr_notification).data.as_ptr(), sample_size);
            let data_types = symbols_and_data_types.data_types();
            let (symbol_info, data_type_info) =
                match symbols_and_data_types.get_symbol_and_data_type(&registered_symbol.name) {
                    Ok(sdt) => sdt,
                    Err(_) => return,
                };
            let variable =
                match Variable::from_bytes(data_types, symbol_info, data_type_info, data_slice) {
                    Ok(v) => v,
                    Err(_) => return,
                };
            (registered_symbol.callback)(&registered_symbol.name, variable);
        }
    }
}

impl AdsTransmissionMode {
    fn to_beckhoff(&self) -> i32 {
        match self {
            Self::None => beckhoff::nAdsTransMode_ADSTRANS_NOTRANS,
            Self::ClientCycle => beckhoff::nAdsTransMode_ADSTRANS_CLIENTCYCLE,
            Self::ClientOnChange => beckhoff::nAdsTransMode_ADSTRANS_CLIENTONCHA,
            Self::Cyclic => beckhoff::nAdsTransMode_ADSTRANS_SERVERCYCLE,
            Self::OnChange => beckhoff::nAdsTransMode_ADSTRANS_SERVERONCHA,
        }
    }
}

fn time_to_beckhoff(time: &Option<Time>) -> Result<u32> {
    match time {
        None => Ok(0),
        Some(Time::Seconds(s)) => Ok(*s as u32 * 10000000),
        Some(Time::MilliSeconds(ms)) => {
            if *ms > 429496 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Millisecond value is too big",
                ));
            }
            Ok(ms * 10000)
        }
        Some(Time::MicroSeconds(us)) => {
            if *us > 429496729 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Microsecond value is too big",
                ));
            }
            Ok(us * 10)
        }
        Some(Time::NanoSeconds(ns)) => {
            if *ns % 100 != 0 {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Nanosecond count must be a multiple of 100",
                ));
            }
            Ok(ns / 100)
        }
    }
}
