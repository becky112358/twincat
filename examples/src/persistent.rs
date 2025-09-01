use std::io::Result;

use twincat::Client;

pub fn persistent(client: &Client) -> Result<()> {
    let persistent_variables = client.symbols_and_data_types().persistent();

    let expected = vec![
        String::from("garden.plants"),
        String::from("main.kitchen.name"),
        String::from("main.kitchen.fridge"),
        String::from("main.dining_room.name"),
        String::from("main.living_room.name"),
        String::from("main.bedroom[0].name"),
        String::from("main.bedroom[1].name"),
        String::from("main.bedroom[2].name"),
        String::from("main.bedroom[3].name"),
        String::from("main.bathroom[0].name"),
    ];

    for variable in &persistent_variables {
        assert!(expected.contains(variable));
    }
    for variable in &expected {
        assert!(persistent_variables.contains(variable));
    }

    Ok(())
}
