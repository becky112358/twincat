use std::io::Result;

use twincat::{path_verify, Client, Variable as V};

pub fn arrays(client: &Client) -> Result<()> {
    println!("{:?}", client.get_value("garden.plants[3]")?);

    client.set_value("garden.plants[3]", V::I16(4))?;
    assert_eq!(V::I16(4), client.get_value("garden.plants[3]")?);

    client.set_value("garden.plants[3]", V::I16(2))?;
    assert_eq!(V::I16(2), client.get_value("garden.plants[3]")?);

    client.set_value("garden.vegetable_plot_at_front[3][2][5]", V::I16(3))?;
    assert_eq!(
        V::I16(3),
        client.get_value("garden.vegetable_plot_at_front[3][2][5]")?
    );

    client.set_value("garden.vegetable_plot_at_back[2,1,0]", V::I16(2))?;
    assert_eq!(
        V::I16(2),
        client.get_value("garden.vegetable_plot_at_back[2,1,0]")?
    );

    println!("{:?}", client.get_value("garden.vegetable_plot_at_front")?);

    println!("{:?}", client.get_value("garden.vegetable_plot_at_back")?);

    get_vegetable_plot_front_0_accessors(client)?;
    get_vegetable_plot_front_1_accessor(client, 3)?;
    get_vegetable_plot_front_2_accessors(client, 3, 1)?;
    get_vegetable_plot_front_3_accessors(client, 3, 1, 4)?;

    let turnips = V::Array(vec![
        V::I16(2),
        V::I16(1),
        V::I16(0),
        V::I16(1),
        V::I16(2),
        V::I16(3),
        V::I16(4),
        V::I16(5),
    ]);
    client.set_value("garden.vegetable_plot_at_front[1][0]", turnips.clone())?;
    assert_eq!(
        turnips,
        client.get_value("garden.vegetable_plot_at_front[1][0]")?
    );

    Ok(())
}

#[path_verify(twincat::Client::builder().connect().unwrap())]
fn get_vegetable_plot_front_0_accessors(client: &Client) -> Result<V> {
    client.get_value("garden.vegetable_plot_at_front")
}

#[path_verify(twincat::Client::builder().connect().unwrap(); [0, 1, 2, 3])]
fn get_vegetable_plot_front_1_accessor(client: &Client, index: usize) -> Result<V> {
    client.get_value(format!("garden.vegetable_plot_at_front[{index}]"))
}

#[path_verify(twincat::Client::builder().connect().unwrap(); [0, 1, 2, 3]; [0, 1, 2])]
fn get_vegetable_plot_front_2_accessors(
    client: &Client,
    index0: usize,
    index1: usize,
) -> Result<V> {
    client.get_value(format!(
        "garden.vegetable_plot_at_front[{index0}][{index1}]"
    ))
}

#[path_verify(twincat::Client::builder().connect().unwrap(); [0, 1, 2, 3]; [0, 1, 2]; [0, 1, 2, 3, 4, 5, 6, 7])]
fn get_vegetable_plot_front_3_accessors(
    client: &Client,
    index0: usize,
    index1: usize,
    index2: usize,
) -> Result<V> {
    client.get_value(format!(
        "garden.vegetable_plot_at_front[{index0}][{index1}][{index2}]"
    ))
}