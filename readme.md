## Tools to interact with TwinCAT using ADS

- Get & Set variable values

### Example
```
use std::io::Result;

use twincat::{path_verify, Client};

fn main() -> Result<()> {
    let client = Client::builder::connect();

    set_room_luminosity(&client, "living_room", 687)?;
    let luminosity_lumens = get_room_luminosity(&client, "living_room")?;
}

#[path_verify(twincat::Client::builder().connect().unwrap(); ALL_ROOMS)]
fn get_room_luminosity(client: &Client, room: &str) -> Result<u16> {
    match client.get_value(format!("MAIN.{room}.actual_luminosity_lumens"))? {
        V::U16(inner) => Ok(inner),
        x => Err(Error::new(
            ErrorKind::Other,
            format!("Unexpected variable type {x:?}"),
        )),
    }
}

#[path_verify(twincat::Client::builder().connect().unwrap(); ALL_ROOMS; [0, 10, 20, 60, 100, 512, 1000, 2856])]
fn set_room_luminosity(client: &Client, room: &str, luminosity: u16) -> Result<()> {
    client.set_value(
        format!("MAIN.{room}.actual_luminosity_lumens"),
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
