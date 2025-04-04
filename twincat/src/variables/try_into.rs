use std::io::{Error, ErrorKind, Result};

use super::Variable;

impl TryInto<bool> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<bool> {
        match self {
            Self::Bool(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected bool, got {other:?}"),
            )),
        }
    }
}

impl TryInto<i8> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<i8> {
        match self {
            Self::I8(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected i8, got {other:?}"),
            )),
        }
    }
}

impl TryInto<i16> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<i16> {
        match self {
            Self::I16(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected i16, got {other:?}"),
            )),
        }
    }
}

impl TryInto<i32> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<i32> {
        match self {
            Self::I32(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected i32, got {other:?}"),
            )),
        }
    }
}

impl TryInto<i64> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<i64> {
        match self {
            Self::I64(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected i64, got {other:?}"),
            )),
        }
    }
}

impl TryInto<u8> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<u8> {
        match self {
            Self::U8(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected u8, got {other:?}"),
            )),
        }
    }
}

impl TryInto<u16> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<u16> {
        match self {
            Self::U16(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected u16, got {other:?}"),
            )),
        }
    }
}

impl TryInto<u32> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<u32> {
        match self {
            Self::U32(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected u32, got {other:?}"),
            )),
        }
    }
}

impl TryInto<u64> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<u64> {
        match self {
            Self::U64(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected u64, got {other:?}"),
            )),
        }
    }
}

impl TryInto<f32> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<f32> {
        match self {
            Self::F32(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected f32, got {other:?}"),
            )),
        }
    }
}

impl TryInto<f64> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<f64> {
        match self {
            Self::F64(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected f64, got {other:?}"),
            )),
        }
    }
}

impl TryInto<String> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<String> {
        match self {
            Self::String(inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected String, got {other:?}"),
            )),
        }
    }
}

impl TryInto<Vec<Variable>> for Variable {
    type Error = Error;

    fn try_into(self) -> Result<Vec<Variable>> {
        match self {
            Self::Array(_, inner) => Ok(inner),
            other => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Expected Vec<>, got {other:?}"),
            )),
        }
    }
}
