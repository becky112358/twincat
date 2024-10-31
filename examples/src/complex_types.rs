use std::io::Result;

use twincat::Client;

pub fn complex_types(client: &Client) -> Result<()> {
    println!("{:?}", client.get_value("MAIN.bedroom")?);

    println!("{:?}", client.get_value("MAIN.kitchen.fridge")?);

    println!("{:?}", client.get_value("MAIN.living_room")?);

    Ok(())
}
