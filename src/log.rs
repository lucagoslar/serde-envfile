#[cfg(feature = "debug")]
macro_rules! debug {
  ($fmt:expr $(, $arg:expr)*) => {
    println!($fmt $(, $arg)*);
  };
}

#[cfg(not(feature = "debug"))]
macro_rules! debug {
    ($fmt:expr $(, $arg:expr)*) => {
      format_args!($fmt $(, $arg)*)
    };
}

pub(crate) use debug;
