use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

/// Flexible representation of environment variables.
///
/// # Example
///
/// ```
/// use serde_envfile::{Value, Error, from_str};
///
/// fn flexible_example() -> Result<(), Error> {
///     let envfile = "HELLO=WORLD";
///
///     let value: Value = from_str(envfile)?;
///     println!("{:?}", value);
///     
///     Ok(())
/// }
/// ```
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct Value {
    #[serde(flatten)]
    inner: HashMap<String, String>,
}

impl Default for Value {
    fn default() -> Self {
        Self::new()
    }
}

impl Value {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

impl Deref for Value {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Value {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_str, to_string};

    #[test]
    fn to_env_test() {
        let mut env = Value::new();
        env.insert("serde_envfile".into(), "HELLO WORLD".into());

        let s = to_string(&env).unwrap();
        let expected = "SERDE_ENVFILE=\"HELLO WORLD\"";
        assert_eq!(expected, s);

        let d: Value = from_str(&s).unwrap();

        assert_eq!(env, d);
    }
}
