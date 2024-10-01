use std::io::{Error, ErrorKind, Result};

use twincat::{Client, Variable as V};

fn main() -> Result<()> {
    let client = Client::builder()
        .with_ams_address([192, 168, 220, 62, 1, 1])
        .connect()?;

    println!("{:#?}", client.symbols());

    println!("{:?}", client.get_value("HOUSE.ADDRESS")?);

    println!("{:?}", client.get_value("HOUSE.N_BATHROOMS")?);

    println!("{}", get_room_luminosity(&client, "dining_room")?);
    set_room_luminosity(&client, "dining_room", 768)?;
    println!("{}", get_room_luminosity(&client, "dining_room")?);

    println!("{:?}", client.get_value("MAIN.bedroom[2].is_occupied")?);

    println!("{:?}", client.get_value("garden.plants[3]")?);
    client.set_value("garden.plants[3]", V::I16(4))?;
    println!("{:?}", client.get_value("garden.plants[3]")?);
    client.set_value("garden.plants[3]", V::I16(2))?;
    println!("{:?}", client.get_value("garden.plants[3]")?);

    client.set_value("garden.vegetable_plot_at_front[3][2][5]", V::I16(3))?;
    println!(
        "{:?}",
        client.get_value("garden.vegetable_plot_at_front[3][2][5]")?
    );

    client.set_value("garden.vegetable_plot_at_back[2,1,0]", V::I16(2))?;
    println!(
        "{:?}",
        client.get_value("garden.vegetable_plot_at_back[2,1,0]")?
    );

    Ok(())
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
