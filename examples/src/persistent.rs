use std::io::Result;

use twincat::Client;

pub fn persistent(client: &Client) -> Result<()> {
    let persistent_variables = client.symbols_and_data_types().persistent();

    let expected = vec![
        String::from("garden.plants"),
        String::from("MAIN.kitchen.name"),
        String::from("MAIN.kitchen.fridge"),
        String::from("MAIN.dining_room.name"),
        String::from("MAIN.living_room.name"),
        String::from("MAIN.bedroom[0].name"),
        String::from("MAIN.bedroom[1].name"),
        String::from("MAIN.bedroom[2].name"),
        String::from("MAIN.bedroom[3].name"),
        String::from("MAIN.bathroom[0].name"),
    ];

    for variable in &persistent_variables {
        assert!(expected.contains(variable));
    }
    for variable in &expected {
        assert!(persistent_variables.contains(variable));
    }

    Ok(())
}
