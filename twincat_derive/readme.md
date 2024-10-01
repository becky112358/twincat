
## Usage

Apply this macro to all functions which access a PLC variable through ADS. Give as arguments the client definition, followed by the full range of each function argument. This macro writes a test which verifies that the PLC variable path is valid.

## Example

```
#[ads_path_verify(crate::client::connect().unwrap(); 0..MAX; ALL_MODES)]
fn function_which_accesses_plc_variable(client: &mut Client, var0: usize, var1: Mode) -> Result<()> {
    client.set_value(
        &format!("Main.some_thing[{var0}].another_thing"),
        var1,
    )
}
```
