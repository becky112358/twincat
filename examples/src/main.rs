use std::io::Result;

use twincat::Client;

fn main() -> Result<()> {
    let client = Client::builder()
        .with_ams_address([192, 168, 220, 62, 1, 1])
        .connect()?;

    println!("{:#?}", client.symbols());

    Ok(())
}
