## Tools to interact with TwinCAT using ADS

- Get & Set variable values

### Example
```
use std::io::Result;

use twincat::Client;

fn main() -> Result<()> {
    let client = Client::builder::connect();

    set_room_luminosity(&client, "living_room", 687)?;
    let luminosity_lumens = get_room_luminosity(&client, "living_room")?;
}

fn get_room_luminosity(client: &Client, room: &str) -> Result<u16> {
    match client.get_value(format!("MAIN.{room}.actual_luminosity_lumens"))? {
        V::U16(inner) => Ok(inner),
        x => Err(Error::new(
            ErrorKind::Other,
            format!("Unexpected variable type {x:?}"),
        )),
    }
}

fn set_room_luminosity(client: &Client, room: &str, luminosity: u16) -> Result<()> {
    client.set_value(
        format!("MAIN.{room}.actual_luminosity_lumens"),
        V::U16(luminosity),
    )?;
    Ok(())
}
```
