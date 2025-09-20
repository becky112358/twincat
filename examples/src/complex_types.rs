use std::io::Result;

use twincat::{Client, StartIndex, Variable as V};

pub fn complex_types(client: &Client) -> Result<()> {
    println!("{:?}", client.get_value("main.bedroom")?);

    println!("{:?}", client.get_value("main.kitchen.fridge")?);

    println!("{:?}", client.get_value("main.living_room")?);

    client.set_value(
        "main.kitchen.fridge",
        V::Struct(vec![
            (
                String::from("drawer"),
                V::Array(
                    StartIndex::Start,
                    vec![V::I16(7), V::I16(6), V::I16(5), V::I16(4)],
                ),
            ),
            (
                String::from("middle_shelf"),
                V::Array(
                    StartIndex::Start,
                    vec![
                        V::I16(0),
                        V::I16(1),
                        V::I16(2),
                        V::I16(3),
                        V::I16(4),
                        V::I16(5),
                        V::I16(6),
                        V::I16(7),
                    ],
                ),
            ),
            (
                String::from("top_shelf"),
                V::Array(
                    StartIndex::Start,
                    vec![
                        V::I16(7),
                        V::I16(6),
                        V::I16(5),
                        V::I16(4),
                        V::I16(3),
                        V::I16(2),
                        V::I16(1),
                        V::I16(0),
                    ],
                ),
            ),
            (
                String::from("bottom_shelf"),
                V::Array(
                    StartIndex::Start,
                    vec![
                        V::I16(0),
                        V::I16(2),
                        V::I16(4),
                        V::I16(6),
                        V::I16(7),
                        V::I16(5),
                        V::I16(3),
                        V::I16(1),
                    ],
                ),
            ),
            (
                String::from("door_shelf"),
                V::Array(
                    StartIndex::Start,
                    vec![V::I16(3), V::I16(2), V::I16(1), V::I16(0)],
                ),
            ),
        ]),
    )?;

    assert_eq!(
        client.get_value("main.kitchen.fridge.middle_shelf[3]")?,
        V::I16(3)
    );
    assert_eq!(
        client.get_value("main.kitchen.fridge.bottom_shelf[7]")?,
        V::I16(1)
    );

    assert!(client
        .set_value(
            "main.kitchen.fridge",
            V::Struct(vec![
                (
                    String::from("top_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("middle_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("bottom_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("door_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 4]),
                ),
                (
                    String::from("drawer"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 4]),
                ),
            ]),
        )
        .is_ok());

    // Struct with gaps
    assert!(client
        .set_value(
            "main.kitchen.fridge",
            V::Struct(vec![
                (
                    String::from("top_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("bottom_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
            ]),
        )
        .is_err());

    // Struct with field set multiple times
    assert!(client
        .set_value(
            "main.kitchen.fridge",
            V::Struct(vec![
                (
                    String::from("top_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("middle_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("bottom_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("top_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("drawer"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 4]),
                ),
                (
                    String::from("door_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 4]),
                ),
            ]),
        )
        .is_err());

    // Struct with non-existent fields
    assert!(client
        .set_value(
            "main.kitchen.fridge",
            V::Struct(vec![
                (
                    String::from("top_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("middle_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("bottom_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("another_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 8]),
                ),
                (
                    String::from("drawer"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 4]),
                ),
                (
                    String::from("door_shelf"),
                    V::Array(StartIndex::Start, vec![V::I16(0); 4]),
                ),
            ]),
        )
        .is_err());

    Ok(())
}
