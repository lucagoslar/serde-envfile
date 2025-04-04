use std::path::Path;

use crate::de::{from_env_inner, from_file_inner, from_str_inner};
use crate::error::Result;
use crate::ser::{to_file_inner, to_string_inner};
use serde::{de, ser};

/// Helper structure to work with prefixes more efficiently.
/// Instantiable through [prefixed].
pub struct Prefixed<'a>(&'a str);

impl<'a> Prefixed<'a> {
    pub fn from_env<T>(&self) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        from_env_inner::<T>(Some(self.0))
    }

    pub fn from_str<T>(&self, input: &'a str) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        from_str_inner::<T>(Some(self.0), input)
    }

    pub fn from_file<T>(&self, path: &Path) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        from_file_inner::<T>(Some(self.0), path)
    }

    pub fn to_string<T>(&self, v: &T) -> Result<String>
    where
        T: ser::Serialize,
    {
        to_string_inner(Some(self.0), v)
    }

    pub fn to_file<T>(&self, path: &Path, v: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        to_file_inner(Some(self.0), path, v)
    }
}

/// Instantiates [Prefixed] from which values can be both serialized and deserialized.
pub fn prefixed(prefix: &str) -> Prefixed {
    Prefixed(prefix)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    use crate::Value;

    #[test]
    fn ser_prefix_test() {
        let mut v = Value::new();
        v.insert("hello".into(), "world".into());

        let s = prefixed("serde_envfile_").to_string(&v).unwrap();
        let expected = "SERDE_ENVFILE_HELLO=\"world\"";
        assert_eq!(expected, s);
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Config {
        hello: String,
    }

    #[test]
    fn de_prefix_test() {
        let env = "SERDE_ENVFILE_HELLO=\"world\"";
        let v = prefixed("serde_envfile_").from_str::<Value>(env).unwrap();
        let mut expected = Value::new();
        expected.insert("hello".into(), "world".into());
        assert_eq!(expected, v);
    }
}
