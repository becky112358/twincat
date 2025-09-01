use std::io::Result;

use twincat::Client;

pub fn complex_types(client: &Client) -> Result<()> {
    println!("{:?}", client.get_value("main.bedroom")?);

    println!("{:?}", client.get_value("main.kitchen.fridge")?);

    println!("{:?}", client.get_value("main.living_room")?);

    Ok(())
}
