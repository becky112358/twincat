use std::io::Result;

use twincat::Client;

fn main() -> Result<()> {
    let _client = Client::builder()
        .with_ams_address([192, 168, 220, 62, 1, 1])
        .connect()?;

    Ok(())
}
