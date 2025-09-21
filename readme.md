## Tools to interact with TwinCAT using ADS

- Get & Set the ADS state
- Get & Set variable values
- Get all `PERSISTENT` variables
- Get all input (`%I*`), output (`%Q*`) and flag (`%M*`) variables
- Request notifications for variable changes
- Verify an ADS path and its associated variable

### Example
```
use std::io::{Error, Result};

use twincat::{path_verify, Client, State, Variable as V};

fn main() -> Result<()> {
    let client = Client::builder::connect()?;

    client.set_ads_state(State::Run)?;

    set_room_luminosity(&client, "living_room", 687)?;
    let luminosity_lumens = get_room_luminosity(&client, "living_room")?;

    Ok(())
}

#[path_verify(twincat::Client::builder().connect().unwrap(); ALL_ROOMS)]
fn get_room_luminosity(client: &Client, room: &str) -> Result<u16> {
    match client.get_value(format!("main.{room}.actual_luminosity_lumens"))? {
        V::U16(inner) => Ok(inner),
        x => Err(Error::other(format!("Unexpected variable type {x:?}"))),
    }
}

#[path_verify(twincat::Client::builder().connect().unwrap(); ALL_ROOMS; [0, 10, 20, 60, 100, 512, 1000, 2856])]
fn set_room_luminosity(client: &Client, room: &str, luminosity: u16) -> Result<()> {
    client.set_value(
        format!("main.{room}.actual_luminosity_lumens"),
        V::U16(luminosity),
    )?;
    Ok(())
}

#[cfg(test)]
const ALL_ROOMS: &[&str] = &[
    "kitchen",
    "living_room",
    "bedroom[0]",
    "bedroom[1]",
    "bathroom[0]",
];
```
