use std::io::{Error, ErrorKind, Result};

pub(super) fn str_array_split(value: &str) -> Result<Vec<String>> {
    if !value.starts_with('[') {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Expected an array, but input does not start with '['\n{value}"),
        ));
    }

    let mut output = Vec::new();
    let mut this = String::new();
    let mut array_count = 0;
    let mut inside_string = false;
    let mut escape = false;
    for (i, c) in value.chars().enumerate() {
        if i == 0 {
            if c == '[' {
                continue;
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Expected an array, but input does not start with '['\n{value}"),
                ));
            }
        } else if i + 1 == value.len() {
            if c == ']' {
                continue;
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Expected an array, but input does not end with ']'\n{value}"),
                ));
            }
        }
        if c == '[' && !inside_string {
            array_count += 1;
        } else if c == ']' && !inside_string {
            array_count -= 1;
        } else if c == '"' && !inside_string {
            inside_string = true;
        } else if c == '"' && inside_string && !escape {
            inside_string = false;
        }

        if c == ',' && array_count == 0 && !inside_string {
            output.push(this);
            this = String::new();
        } else {
            this.push(c);
        }

        if array_count < 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Invalid array: {value}"),
            ));
        }

        escape = c == '\\' && inside_string;
    }

    output.push(this);

    if array_count != 0 || inside_string {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Invalid array: {value}"),
        ));
    }

    Ok(output)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_simple_array() {
        assert_eq!(
            str_array_split("[1,2,3,4,5]").unwrap(),
            vec![
                String::from("1"),
                String::from("2"),
                String::from("3"),
                String::from("4"),
                String::from("5"),
            ]
        )
    }

    #[test]
    fn split_array_with_strings() {
        assert_eq!(
            str_array_split("[\"hello\",\"world\",\"let's \"escape\"!\"]").unwrap(),
            vec![
                String::from("\"hello\""),
                String::from("\"world\""),
                String::from("\"let's \"escape\"!\""),
            ]
        )
    }

    #[test]
    fn split_array_of_arrays() {
        assert_eq!(
            str_array_split("[[1,2,3],[4,5,6],[20,21,22]]").unwrap(),
            vec![
                String::from("[1,2,3]"),
                String::from("[4,5,6]"),
                String::from("[20,21,22]"),
            ]
        )
    }

    #[test]
    fn split_invalid_arrays() {
        assert!(str_array_split("[1,2,3,4").is_err());
        assert!(str_array_split("1,2,3,4]").is_err());
        assert!(str_array_split("[1,2,3]x[4,5,6]").is_err());
        assert!(str_array_split("[1,2,\",x,z]").is_err());
        assert!(str_array_split("[1,2,x,z]").is_ok())
    }
}
