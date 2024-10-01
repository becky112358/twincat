use std::io::{Error, ErrorKind, Result};

use strum_macros::EnumIter;

use super::client::Client;
use super::{beckhoff, result};

#[derive(Debug, EnumIter, PartialEq)]
pub enum State {
    Invalid,
    Idle,
    Reset,
    Init,
    Start,
    Run,
    Stop,
    SaveCfg,
    LoadCfg,
    PowerFailure,
    PowerGood,
    Error,
    Shutdown,
    Suspend,
    Resume,
    Config,
    Reconfig,
    Stopping,
    MaxStates,
}

impl State {
    fn from_beckhoff(state: i32) -> Result<Self> {
        match state {
            beckhoff::nAdsState_ADSSTATE_INVALID => Ok(State::Invalid),
            beckhoff::nAdsState_ADSSTATE_IDLE => Ok(State::Idle),
            beckhoff::nAdsState_ADSSTATE_RESET => Ok(State::Reset),
            beckhoff::nAdsState_ADSSTATE_INIT => Ok(State::Init),
            beckhoff::nAdsState_ADSSTATE_START => Ok(State::Start),
            beckhoff::nAdsState_ADSSTATE_RUN => Ok(State::Run),
            beckhoff::nAdsState_ADSSTATE_STOP => Ok(State::Stop),
            beckhoff::nAdsState_ADSSTATE_SAVECFG => Ok(State::SaveCfg),
            beckhoff::nAdsState_ADSSTATE_LOADCFG => Ok(State::LoadCfg),
            beckhoff::nAdsState_ADSSTATE_POWERFAILURE => Ok(State::PowerFailure),
            beckhoff::nAdsState_ADSSTATE_POWERGOOD => Ok(State::PowerGood),
            beckhoff::nAdsState_ADSSTATE_ERROR => Ok(State::Error),
            beckhoff::nAdsState_ADSSTATE_SHUTDOWN => Ok(State::Shutdown),
            beckhoff::nAdsState_ADSSTATE_SUSPEND => Ok(State::Suspend),
            beckhoff::nAdsState_ADSSTATE_RESUME => Ok(State::Resume),
            beckhoff::nAdsState_ADSSTATE_CONFIG => Ok(State::Config),
            beckhoff::nAdsState_ADSSTATE_RECONFIG => Ok(State::Reconfig),
            beckhoff::nAdsState_ADSSTATE_STOPPING => Ok(State::Stopping),
            beckhoff::nAdsState_ADSSTATE_MAXSTATES => Ok(State::MaxStates),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("{state} does not match an ADS State"),
            )),
        }
    }

    fn to_beckhoff(&self) -> i32 {
        match self {
            State::Invalid => beckhoff::nAdsState_ADSSTATE_INVALID,
            State::Idle => beckhoff::nAdsState_ADSSTATE_IDLE,
            State::Reset => beckhoff::nAdsState_ADSSTATE_RESET,
            State::Init => beckhoff::nAdsState_ADSSTATE_INIT,
            State::Start => beckhoff::nAdsState_ADSSTATE_START,
            State::Run => beckhoff::nAdsState_ADSSTATE_RUN,
            State::Stop => beckhoff::nAdsState_ADSSTATE_STOP,
            State::SaveCfg => beckhoff::nAdsState_ADSSTATE_SAVECFG,
            State::LoadCfg => beckhoff::nAdsState_ADSSTATE_LOADCFG,
            State::PowerFailure => beckhoff::nAdsState_ADSSTATE_POWERFAILURE,
            State::PowerGood => beckhoff::nAdsState_ADSSTATE_POWERGOOD,
            State::Error => beckhoff::nAdsState_ADSSTATE_ERROR,
            State::Shutdown => beckhoff::nAdsState_ADSSTATE_SHUTDOWN,
            State::Suspend => beckhoff::nAdsState_ADSSTATE_SUSPEND,
            State::Resume => beckhoff::nAdsState_ADSSTATE_RESUME,
            State::Config => beckhoff::nAdsState_ADSSTATE_CONFIG,
            State::Reconfig => beckhoff::nAdsState_ADSSTATE_RECONFIG,
            State::Stopping => beckhoff::nAdsState_ADSSTATE_STOPPING,
            State::MaxStates => beckhoff::nAdsState_ADSSTATE_MAXSTATES,
        }
    }
}

impl Client {
    pub fn get_ads_state(&self) -> Result<State> {
        let ptr_address = &mut self.ams_address().to_owned() as *mut beckhoff::AmsAddr;

        let mut ads_state = 0u16;
        let ptr_ads_state = &mut ads_state as *mut u16;

        let mut device_state = 0u16;
        let ptr_device_state = &mut device_state as *mut u16;

        result::process(unsafe {
            beckhoff::AdsSyncReadStateReqEx(
                self.port(),
                ptr_address,
                ptr_ads_state,
                ptr_device_state,
            )
        })?;

        State::from_beckhoff(ads_state as i32)
    }

    pub fn set_ads_state(&self, state: State) -> Result<()> {
        let current_state = self.get_ads_state()?;
        if state == current_state {
            return Ok(());
        }

        let ptr_address = &mut self.ams_address().to_owned() as *mut beckhoff::AmsAddr;

        let u16_state = state.to_beckhoff() as u16;

        result::process(unsafe {
            beckhoff::AdsSyncWriteControlReqEx(
                self.port(),
                ptr_address,
                u16_state,
                0,
                0,
                std::ptr::null_mut(),
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use strum::IntoEnumIterator;

    #[test]
    fn state_to_beckhoff_to_state() {
        for state in State::iter() {
            assert_eq!(state, State::from_beckhoff(state.to_beckhoff()).unwrap());
        }
    }
}
