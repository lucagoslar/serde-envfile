use std::path::Path;

use serde::de;

use crate::{Error, error::Result};

pub fn from_iter<T, Iter>(iter: Iter) -> Result<T>
where
    T: de::DeserializeOwned,
    Iter: IntoIterator<Item = (String, String)>,
{
    from_iter_inner::<T, Iter>(None, iter)
}

pub fn from_iter_inner<T, Iter>(prefix: Option<&str>, iter: Iter) -> Result<T>
where
    T: de::DeserializeOwned,
    Iter: IntoIterator<Item = (String, String)>,
{
    match prefix {
        Some(pref) => {
            envy::prefixed(pref.to_uppercase()).from_iter::<_, T>(iter)
        }
        None => {
            // No prefix provided, use default behavior
            envy::from_iter::<_, T>(iter)
        }
    }.map_err(Error::new)
}

/// Deserialize program-available environment variables into an instance of type `T`.
///
/// # Example
///
/// ```
/// use serde::Deserialize;
/// use serde_envfile::{from_env, Error};
///
/// #[derive(Debug, Deserialize)]
/// struct Test {
///     #[cfg(windows)]
///     #[serde(rename="username")]
///     user: String,
///     #[cfg(not(windows))]
///     user: String,
/// }
///
/// fn from_env_example() -> Result<(), Error> {
///     let t: Test = from_env()?;
///     println!("{:?}", t);
///
///     Ok(())
/// }
/// ```
pub fn from_env<T>() -> Result<T>
where
    T: de::DeserializeOwned,
{
    from_env_inner(None)
}

pub fn from_env_inner<T>(prefix: Option<&str>) -> Result<T>
where
    T: de::DeserializeOwned,
{
    match prefix {
        Some(pref) => {
            envy::prefixed(pref.to_uppercase()).from_env::<T>()
        }
        None => {
            envy::from_env::<T>()
        }
    }.map_err(Error::new)
}

/// Deserialize environment variables from a string into an instance of type `T`.
///
/// # Example
///
/// ```
/// use serde_envfile::{from_str, Value, Error};
///
/// fn from_str_example() -> Result<(), Error> {
///     let e = "HELLO=world";
///     let v: Value = from_str(e)?;
///     println!("{:?}", v);
///
///     Ok(())
/// }
/// ```
pub fn from_str<T>(input: &str) -> Result<T>
where
    T: de::DeserializeOwned,
{
    from_str_inner::<T>(None, input)
}

pub fn from_str_inner<'a, T>(prefix: Option<&'a str>, input: &'a str) -> Result<T>
where
    T: de::DeserializeOwned,
{
    let mut env = Vec::new();

    for pair in dotenvy::from_read_iter(input.as_bytes()) {
        let (key, value) = pair.map_err(Error::new)?;
        env.push((key, value));
    }

    from_iter_inner::<T, _>(prefix, env)
}

/// Deserialize an environment variable file into an instance of type `T`.
///
/// # Example
///
/// ```
/// use std::path::PathBuf;
/// use serde_envfile::{from_file, Value, Error};
///
/// fn from_file_example() -> Result<(), Error> {
///     let v: Value = from_file(&PathBuf::from(".env"))?;
///     println!("{:?}", v);
///
///     Ok(())
/// }
/// ```
pub fn from_file<T>(path: &Path) -> Result<T>
where
    T: de::DeserializeOwned,
{
    from_file_inner(None, path)
}

pub fn from_file_inner<T>(prefix: Option<&str>, path: &Path) -> Result<T>
where
    T: de::DeserializeOwned,
{
    let mut env = Vec::new();

    for pair in dotenvy::from_filename_iter(path)
        .map_err(Error::new)?
    {
        let (key, value) = pair.map_err(Error::new)?;
        env.push((key, value));
    }

    from_iter_inner::<T, _>(prefix, env)
}

#[cfg(test)]
mod tests {
    use std::{
        env::{set_var, vars},
        fs::write,
        io::{SeekFrom, prelude::*},
    };

    use tempfile::NamedTempFile;

    use super::*;
    use crate::Value;

    #[test]
    fn from_env_test() {
        unsafe {
            set_var("SERDE_ENVFILE", "HELLO WORLD");
        }

        let env: Value = from_env().unwrap();

        assert_eq!(env.len(), vars().collect::<Vec<(String, String)>>().len());

        for (key, value) in vars() {
            assert_eq!(&value, env.get(&key.to_lowercase()).unwrap());
        }
    }

    #[test]
    fn from_str_test() {
        let input = "HELLO=world";
        let env: Value = from_str(input).unwrap();

        assert_eq!(env.len(), 1);
        assert_eq!("world", env.get("hello").unwrap());
    }

    #[test]
    fn from_file_test() {
        let input = "HELLO=world";
        let mut file = NamedTempFile::new().unwrap();
        write(file.path(), input).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        let env: Value = from_file(file.path()).unwrap();

        assert_eq!(env.len(), 1);
        assert_eq!("world", env.get("hello").unwrap());
    }
}
