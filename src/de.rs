use std::path::Path;

use serde::de;

use crate::{error::Result, Error};

pub fn from_iter<T, Iter>(iter: Iter) -> Result<T>
where
    T: de::DeserializeOwned,
    Iter: IntoIterator<Item = (String, String)>,
{
    from_iter_inner::<T, Iter>(None, iter)
}

pub fn from_iter_inner<'a, T, Iter>(prefix: Option<&'a str>, iter: Iter) -> Result<T>
where
    T: de::DeserializeOwned,
    Iter: IntoIterator<Item = (String, String)>,
{
    (if prefix.is_some() {
        envy::prefixed(prefix.unwrap().to_uppercase()).from_iter::<_, T>(iter)
    } else {
        envy::from_iter::<_, T>(iter)
    })
    .map_err(|e| Error::new(e))
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

pub fn from_env_inner<'a, T>(prefix: Option<&'a str>) -> Result<T>
where
    T: de::DeserializeOwned,
{
    (if prefix.is_some() {
        envy::prefixed(prefix.unwrap().to_uppercase()).from_env::<T>()
    } else {
        envy::from_env::<T>()
    })
    .map_err(|e| Error::new(e))
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
pub fn from_str<'a, T>(input: &'a str) -> Result<T>
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

    for pair in dotenvy::from_read_iter(input.as_bytes()).into_iter() {
        let (key, value) = pair.map_err(|e| Error::new(e))?;
        env.push((key, value));
    }

    from_iter_inner::<T, _>(prefix, env.into_iter())
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
pub fn from_file<'a, T>(path: &Path) -> Result<T>
where
    T: de::DeserializeOwned,
{
    from_file_inner(None, path)
}

pub fn from_file_inner<'a, T>(prefix: Option<&'a str>, path: &Path) -> Result<T>
where
    T: de::DeserializeOwned,
{
    let mut env = Vec::new();

    for pair in dotenvy::from_filename_iter(path)
        .map_err(|e| Error::new(e))?
        .into_iter()
    {
        let (key, value) = pair.map_err(|e| Error::new(e))?;
        env.push((key, value));
    }

    from_iter_inner::<T, _>(prefix, env.into_iter())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env::{set_var, vars},
        fs::write,
        io::{prelude::*, SeekFrom},
    };
    use tempfile::NamedTempFile;

    use crate::Value;

    #[test]
    fn from_env_test() {
        set_var("SERDE_ENVFILE", "HELLO WORLD");

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

        let env: Value = from_file(&file.path().to_path_buf()).unwrap();

        assert_eq!(env.len(), 1);
        assert_eq!("world", env.get("hello").unwrap());
    }
}
