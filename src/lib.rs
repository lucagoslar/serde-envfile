//! # serde-envfile
//! Built ontop the [`dotenvy`](https://github.com/allan2/dotenvy) and
//! [`envy`](https://github.com/softprops/envy) crates, `serde-envfile`
//! supports both the serialization and the deserialization of environment
//! variables from or to files (`from_file`, `to_file`), strings
//! (`from_str`, `to_string`), or the environment of the application
//! (`from_env`, `to_env`).
//!
//! ## Examples
//! Note that keys are transformed to lowercase during deserialization.
//! With serialization, the contrary is the case.
//! ```no_run
//! use serde::{Deserialize, Serialize};
//! use serde_envfile::{Error, from_str, to_string};
//!
//! #[derive(Debug, Deserialize, Serialize)]
//! struct Test {
//!     hello: String,
//! }
//!
//! fn main() -> Result<(), Error> {
//!     let env = "HELLO=\"WORLD\"";
//!     let test: Test = from_str(env)?;
//!     let env = to_string(&test)?;
//!
//!     println!("{}", env);
//!
//!     Ok(())
//! }
//! ```
//!
//! Introducing the `Value` type, `serde-envfile`, also provides a more flexible approach to working with environment variables.
//! ```no_run
//! use serde_envfile::{to_string, Error, Value};
//!
//! fn main() -> Result<(), Error> {
//!    let mut env = Value::new();
//!    env.insert("hello".into(), "world".into());
//!    let env = to_string(&env)?;
//!
//!    println!("{}", env);
//!
//!    Ok(())
//! }
//! ```

#[doc(hidden)]
pub mod de;
pub(crate) mod error;
pub(crate) mod prefixed;
pub(crate) mod ser;
pub(crate) mod value;

pub use error::Error;

pub use de::from_env;
pub use de::from_file;
pub use de::from_str;

pub use ser::Serializer;
pub use ser::to_file;
pub use ser::to_string;

pub use value::Value;

pub use prefixed::Prefixed;
pub use prefixed::prefixed;
