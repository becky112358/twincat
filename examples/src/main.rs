use std::io::{Error, ErrorKind, Result};
use std::thread;
use std::time::Duration;

use twincat::{path_verify, Client, State, Variable as V};

fn main() -> Result<()> {
    let client = Client::builder()
        .with_ams_address([192, 168, 220, 62, 1, 1])
        .connect()?;

    println!("{:#?}", client.symbols());

    toggle_state(&client)?;

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

    verify_heating(&client)?;

    Ok(())
}

fn toggle_state(client: &Client) -> Result<()> {
    println!("{:?}", client.get_ads_state()?);
    client.set_ads_state(State::Run)?;
    println!("{:?}", client.get_ads_state()?);
    client.set_ads_state(State::Stop)?;
    println!("{:?}", client.get_ads_state()?);
    client.set_ads_state(State::Run)?;
    println!("{:?}", client.get_ads_state()?);

    Ok(())
}

#[path_verify(twincat::Client::builder().with_ams_address([192, 168, 220, 62, 1, 1]).connect().unwrap(); ALL_ROOMS)]
fn get_room_luminosity(client: &Client, room: &str) -> Result<u16> {
    match client.get_value(format!("MAIN.{room}.actual_luminosity_lumens"))? {
        V::U16(inner) => Ok(inner),
        x => Err(Error::new(
            ErrorKind::Other,
            format!("Unexpected variable type {x:?}"),
        )),
    }
}

#[path_verify(twincat::Client::builder().with_ams_address([192, 168, 220, 62, 1, 1]).connect().unwrap(); ALL_ROOMS; [0, 10, 20, 60, 100, 512, 1000, 2856])]
fn set_room_luminosity(client: &Client, room: &str, luminosity: u16) -> Result<()> {
    client.set_value(
        format!("MAIN.{room}.actual_luminosity_lumens"),
        V::U16(luminosity),
    )?;
    Ok(())
}

fn verify_heating(client: &Client) -> Result<()> {
    // Setup
    client.set_ads_state(State::Run)?;
    client.set_value("MAIN.living_room.is_occupied", V::Bool(false))?;
    client.set_value("MAIN.living_room.actual_temperature_oc", V::F32(22.4))?;

    thread::sleep(Duration::from_secs(5));

    // Room is warm => Heating is off
    assert_eq!(
        client.get_value("MAIN.living_room.heating_on")?,
        V::Bool(false)
    );

    client.set_value("MAIN.living_room.actual_temperature_oc", V::F32(12.4))?;

    thread::sleep(Duration::from_secs(5));

    // Room is cold but unoccupied => Heating is off
    assert_eq!(
        client.get_value("MAIN.living_room.heating_on")?,
        V::Bool(false)
    );

    client.set_value("MAIN.living_room.is_occupied", V::Bool(true))?;

    thread::sleep(Duration::from_secs(1));

    // Room is cold and occupied => Heating is on
    assert_eq!(
        client.get_value("MAIN.living_room.heating_on")?,
        V::Bool(true)
    );

    client.set_value("MAIN.living_room.is_occupied", V::Bool(false))?;

    for _ in 0..5 {
        thread::sleep(Duration::from_secs(5));

        // Room is cold and was occupied recently => Heating is on
        assert_eq!(
            client.get_value("MAIN.living_room.heating_on")?,
            V::Bool(true)
        );
    }

    thread::sleep(Duration::from_secs(7));

    // Room is cold but unoccupied => Heating is off
    assert_eq!(
        client.get_value("MAIN.living_room.heating_on")?,
        V::Bool(false)
    );

    // Room is occupied, then briefly unoccupied, then reoccupied => Heating remains on
    client.set_value("MAIN.living_room.is_occupied", V::Bool(true))?;

    thread::sleep(Duration::from_secs(5));

    assert_eq!(
        client.get_value("MAIN.living_room.heating_on")?,
        V::Bool(true)
    );

    client.set_value("MAIN.living_room.is_occupied", V::Bool(false))?;

    thread::sleep(Duration::from_secs(5));

    assert_eq!(
        client.get_value("MAIN.living_room.heating_on")?,
        V::Bool(true)
    );

    client.set_value("MAIN.living_room.is_occupied", V::Bool(true))?;

    thread::sleep(Duration::from_secs(35));

    assert_eq!(
        client.get_value("MAIN.living_room.heating_on")?,
        V::Bool(true)
    );

    Ok(())
}

#[cfg(test)]
const ALL_ROOMS: &[&str] = &[
    "kitchen",
    "dining_room",
    "living_room",
    "bedroom[0]",
    "bedroom[1]",
    "bedroom[2]",
    "bedroom[3]",
    "bathroom[0]",
];
