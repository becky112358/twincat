## Tools to interact with TwinCAT using ADS

### Example
```
use std::io::Result;

use twincat::Client;

fn main() -> Result<()> {
    let _client = Client::builder::connect();
}
```
