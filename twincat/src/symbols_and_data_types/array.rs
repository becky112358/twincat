use std::io::{Error, ErrorKind, Result};
use std::ops::RangeInclusive;
use std::str::FromStr;

pub(super) fn get_base_name(data_type: &str, n_array_accessings: Option<u8>) -> Result<&str> {
    let base_name = match n_array_accessings {
        Some(mut n_remaining_accessings) => {
            let mut remainder = data_type;
            while n_remaining_accessings > 0 {
                match (
                    remainder.find('['),
                    remainder.find(']'),
                    remainder.find(" OF "),
                ) {
                    (Some(i0), Some(i1), Some(i_of)) => {
                        if i0 >= i1 || i1 >= i_of || i_of + 4 > remainder.len() {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                format!("Unexpected data type {remainder}"),
                            ));
                        }
                        let n_commas =
                            remainder[i0..i1].chars().filter(|c| *c == ',').count() as u8;
                        let n_accessings = n_commas + 1;
                        if n_accessings > n_remaining_accessings {
                            return Err(Error::new(
                                ErrorKind::InvalidInput,
                                "Cannot access the middle of arrays of type [a..b,y..c]",
                            ));
                        }
                        n_remaining_accessings -= n_accessings;
                        remainder = remainder[i_of + 4..].trim();
                    }
                    _ => {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "Out-of-bounds error: Too many array accessors!",
                        ))
                    }
                }
            }
            remainder
        }
        None => match data_type.rfind(" OF ") {
            Some(start) => data_type[start + 4..].trim(),
            None => data_type,
        },
    };

    Ok(base_name)
}

pub(super) fn get_ranges(input: &str) -> Result<Vec<RangeInclusive<i32>>> {
    let mut output = Vec::new();

    let mut remainder = input;

    while remainder.contains("ARRAY") {
        let square_start = match remainder.find('[') {
            Some(i) => i,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot parse array dimensions: Cannot find '[' in {input}"),
                ))
            }
        };
        let square_end = match remainder.find(']') {
            Some(i) => i,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot parse array dimensions: Cannot find ']' in {input}"),
                ))
            }
        };

        if square_start > square_end {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Cannot parse array dimensions: '[' comes after ']' in {input}"),
            ));
        }

        let dimensions = remainder[square_start + 1..square_end]
            .split(',')
            .collect::<Vec<&str>>();

        for dimension in dimensions {
            let mid = match dimension.find("..") {
                Some(i) => i,
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Cannot parse array dimensions: Cannot find \"..\" in {input}"),
                    ))
                }
            };
            let start_str = &dimension[..mid];
            let end_str = &dimension[mid + 2..];
            let start = match i32::from_str(start_str) {
                Ok(x) => x,
                Err(e) => return Err(Error::new(ErrorKind::InvalidData, format!("Cannot parse array dimensions: {input} contains invalid start dimension {start_str} ({e})"))),
            };
            let end = match i32::from_str(end_str) {
                Ok(x) => x,
                Err(e) => return Err(Error::new(ErrorKind::InvalidData, format!("Cannot parse array dimensions: {input} contains invalid end dimension {end_str} ({e})"))),
            };

            if start > end {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid array dimensions: {start} > {end} in {input}"),
                ));
            }

            output.push(RangeInclusive::new(start, end));
        }

        remainder = &remainder[square_end + 1..];
    }

    Ok(output)
}

pub(super) fn count_accessors(input: &str) -> u8 {
    let mut output = 0;

    let mut inside_bracket = false;

    for c in input.chars().rev() {
        if c == ']' {
            inside_bracket = true;
            output += 1;
        } else if inside_bracket && c == ',' {
            output += 1;
        } else if c == '[' {
            inside_bracket = false;
        } else if c == '.' {
            break;
        }
    }

    output
}

pub(super) fn trim_accessors(input: &str) -> String {
    match input.split_once('[') {
        Some((start, _)) => start.to_string(),
        None => input.to_string(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_get_base_name() {
        assert_eq!(get_base_name("UINT", None).unwrap(), "UINT");
        assert!(get_base_name("UINT", Some(1)).is_err());

        assert_eq!(get_base_name("ARRAY [3..7] OF UINT", None).unwrap(), "UINT");
        assert_eq!(get_base_name("ARRAY [3..7] OF UINT", Some(1)).unwrap(), "UINT");
        assert!(get_base_name("ARRAY [3..7] OF UINT", Some(2)).is_err());

        assert_eq!(get_base_name("ARRAY [-6..2] OF ARRAY [3..7] OF UINT", None).unwrap(), "UINT");
        assert_eq!(get_base_name("ARRAY [-6..2] OF ARRAY [3..7] OF UINT", Some(1)).unwrap(), "ARRAY [3..7] OF UINT");
        assert_eq!(get_base_name("ARRAY [-6..2] OF ARRAY [3..7] OF UINT", Some(2)).unwrap(), "UINT");
        assert!(get_base_name("ARRAY [-6..2] OF ARRAY [3..7] OF UINT", Some(3)).is_err());
    }

    #[test]
    #[rustfmt::skip]
    fn get_base_name_of_array_with_comma() {
        assert_eq!(get_base_name("ARRAY [0..1,1..2] OF USINT", None).unwrap(), "USINT");
        assert!(get_base_name("ARRAY [0..1,1..2] OF USINT", Some(1)).is_err());
        assert_eq!(get_base_name("ARRAY [0..1,1..2] OF USINT", Some(2)).unwrap(), "USINT");
        assert!(get_base_name("ARRAY [0..1,1..2] OF USINT", Some(3)).is_err());

        assert_eq!(get_base_name("ARRAY [0..1,1..2,2..3] OF ARRAY [3..5] OF ARRAY [2..4,4..5] OF USINT", None).unwrap(), "USINT");
        assert!(get_base_name("ARRAY [0..1,1..2,2..3] OF ARRAY [3..5] OF ARRAY [2..4,4..5] OF USINT", Some(1)).is_err());
        assert!(get_base_name("ARRAY [0..1,1..2,2..3] OF ARRAY [3..5] OF ARRAY [2..4,4..5] OF USINT", Some(2)).is_err());
        assert_eq!(get_base_name("ARRAY [0..1,1..2,2..3] OF ARRAY [3..5] OF ARRAY [2..4,4..5] OF USINT", Some(3)).unwrap(), "ARRAY [3..5] OF ARRAY [2..4,4..5] OF USINT");

        assert_eq!(get_base_name("ARRAY [0..1,1..2,2..3] OF ARRAY [3..5] OF ARRAY [2..4,4..5] OF USINT", Some(4)).unwrap(), "ARRAY [2..4,4..5] OF USINT");

        assert!(get_base_name("ARRAY [0..1,1..2,2..3] OF ARRAY [3..5] OF ARRAY [2..4,4..5] OF USINT", Some(5)).is_err());
        assert_eq!(get_base_name("ARRAY [0..1,1..2,2..3] OF ARRAY [3..5] OF ARRAY [2..4,4..5] OF USINT", Some(6)).unwrap(), "USINT");
        assert!(get_base_name("ARRAY [0..1,1..2,2..3] OF ARRAY [3..5] OF ARRAY [2..4,4..5] OF USINT", Some(7)).is_err());
    }

    #[test]
    fn test_get_ranges() {
        assert_eq!(get_ranges("UINT").unwrap(), vec![]);
        assert_eq!(
            get_ranges("ARRAY [3..7] OF UINT").unwrap(),
            vec![RangeInclusive::new(3, 7)]
        );
        assert_eq!(
            get_ranges("ARRAY [-6..2] OF ARRAY [3..7] OF UINT").unwrap(),
            vec![RangeInclusive::new(-6, 2), RangeInclusive::new(3, 7)]
        );
        assert_eq!(
            get_ranges(
                "ARRAY [-6..2,1..3,-8..-4] OF ARRAY [3..7] OF ARRAY [1..2,2..3,3..4] OF UINT"
            )
            .unwrap(),
            vec![
                RangeInclusive::new(-6, 2),
                RangeInclusive::new(1, 3),
                RangeInclusive::new(-8, -4),
                RangeInclusive::new(3, 7),
                RangeInclusive::new(1, 2),
                RangeInclusive::new(2, 3),
                RangeInclusive::new(3, 4)
            ]
        );

        assert!(get_ranges("ARRAY [..7] OF UINT").is_err());
    }

    #[test]
    fn test_count_accessors() {
        assert_eq!(count_accessors("my_value"), 0);
        assert_eq!(count_accessors("my_value[-8]"), 1);
        assert_eq!(count_accessors("my_value[3][5]"), 2);
        assert_eq!(count_accessors("my_value[3,5]"), 2);
        assert_eq!(count_accessors("my_value[-8][3,5]"), 3);
    }

    #[test]
    fn test_trim_accessors() {
        assert_eq!(trim_accessors("my_value"), String::from("my_value"));
        assert_eq!(trim_accessors("my_value[-8]"), String::from("my_value"));
        assert_eq!(trim_accessors("my_value[3][5]"), String::from("my_value"));
    }
}
