use std::io::Result;

use twincat::{AdsTransmissionMode, Client, Time, Variable as V};

pub fn notifications(client: &Client) -> Result<()> {
    const ADS_PATH: &str = "garden.vegetable_plot_at_front[3][2][5]";

    let notification_handle = client.request_notifications(
        ADS_PATH.to_string(),
        AdsTransmissionMode::OnChange,
        None,
        Some(Time::Seconds(1)),
        callback,
    )?;

    client.set_value(ADS_PATH, V::I16(2))?;

    client.set_value("garden.vegetable_plot_at_front[3][2][1]", V::I16(0))?;

    std::thread::sleep(std::time::Duration::from_secs(5));

    client.delete_notification_with_handle(notification_handle)?;

    client.set_value(ADS_PATH, V::I16(3))?;

    Ok(())
}

fn callback(value_name: &str, variable: V) {
    println!("Value changed!");
    println!("{value_name} is now {variable:?}");
}
