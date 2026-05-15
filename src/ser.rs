use std::{fs::File, path::Path};

use serde::ser::Serialize as _;

use super::error::{Error, Result};

cfg_if::cfg_if! {
    if #[cfg(feature = "debug")] {
        use log::debug;
    } else {
        #[allow(unused_macros)]
        macro_rules! debug {
            ($fmt:expr $(, $arg:expr)*) => {};
        }
    }
}

/// A serializer to transform Rust data into environment variables.
pub struct Serializer<W> {
    writer: W,
    base_prefix: Option<String>,
    prefix: Vec<String>,
    key: bool,
    seq_state: SeqState,
    pending_key: bool,
    first: bool,
}

enum SeqState {
    NotSeq,
    First,
    Rest,
}

impl<W: std::io::Write> Serializer<W> {
    fn new<T: AsRef<str>>(prefix: Option<T>, writer: W) -> Self {
        Self {
            writer: writer,
            base_prefix: prefix
                .map(|s| s.as_ref().to_uppercase())
                .filter(|s| !s.is_empty()),
            prefix: Vec::new(),
            key: false,
            seq_state: SeqState::NotSeq,
            pending_key: false,
            first: true,
        }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    fn write_pending_key(&mut self) -> Result<()> {
        if self.pending_key {
            let key = match &self.base_prefix {
                Some(s) => format!("{}{}", s, self.prefix.join("_")),
                None => self.prefix.join("_"),
            };
            if self.first {
                self.first = false
            } else {
                self.writer.write_all(b"\n")?;
            }
            self.writer.write_all(key.as_bytes())?;
            self.writer.write_all(b"=")?;
            self.pending_key = false;
        };
        Ok(())
    }
}

/// Serialize data into an environment variable string.
///
/// # Example
///
/// ```
/// use serde_envfile::{Value, to_string};
///
/// let value = Value::from_iter([("KEY", "VALUE")]);
///
/// let value = to_string(&value).expect("Failed to serialize to string");
/// println!("{}", value);
/// ```
pub fn to_string<T>(v: &T) -> Result<String>
where
    T: serde::ser::Serialize,
{
    to_string_inner(None, v)
}

pub fn to_string_inner<T>(prefix: Option<&str>, v: &T) -> Result<String>
where
    T: serde::ser::Serialize,
{
    let mut vec = Vec::with_capacity(128);
    let mut serializer = Serializer::new(prefix, &mut vec);
    v.serialize(&mut serializer)?;
    // Safe because we do not emit invalid UTF-8.
    Ok(unsafe { String::from_utf8_unchecked(vec) })
}

/// Serialize data to a writer that implements `std::io::Write`.
///
/// # Example
///
/// ```
/// use std::io::Cursor;
/// use serde_envfile::{Value, to_writer};
///
/// let value = Value::from_iter([("KEY", "VALUE")]);
///
/// let mut writer = Cursor::new(Vec::new());
/// to_writer(&mut writer, &value).expect("Failed to serialize to writer");
///
/// let output = String::from_utf8(writer.into_inner()).expect("Invalid UTF-8 sequence");
/// println!("{}", output);
/// ```
pub fn to_writer<W, T>(writer: W, v: &T) -> Result<()>
where
    W: std::io::Write,
    T: serde::ser::Serialize,
{
    to_writer_inner(None, writer, v)
}

pub(crate) fn to_writer_inner<W, T>(prefix: Option<&str>, writer: W, v: &T) -> Result<()>
where
    W: std::io::Write,
    T: serde::ser::Serialize,
{
    let mut serializer = Serializer::new(prefix, writer);
    v.serialize(&mut serializer)
}

/// Serialize data into an environment variable file.
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use serde_envfile::{Value, to_file};
///
/// let value = Value::from_iter([("KEY", "VALUE")]);
/// let path = PathBuf::from(".env");
///
/// to_file(&path, &value).expect("Failed to serialize to file");
/// ```
pub fn to_file<P, T>(path: P, v: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: serde::ser::Serialize,
{
    to_file_inner(None, path, v)
}

pub fn to_file_inner<P, T>(prefix: Option<&str>, path: P, v: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: serde::ser::Serialize,
{
    let file = File::create(path).map_err(Error::new)?;
    to_writer_inner(prefix, file, v)
}

impl<W> serde::ser::Serializer for &mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        debug!("serialize bool: {}", v);
        self.write_pending_key()?;
        self.writer.write_all(if v { b"true" } else { b"false" })?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        debug!("serialize i8: {}", v);
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        debug!("serialize i16: {}", v);
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        debug!("serialize i32: {}", v);
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        debug!("serialize i64: {}", v);
        self.write_pending_key()?;
        self.writer.write_all(v.to_string().as_bytes())?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        debug!("serialize u8: {}", v);
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        debug!("serialize u16: {}", v);
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        debug!("serialize u32: {}", v);
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        debug!("serialize u64: {}", v);
        self.write_pending_key()?;
        self.writer.write_all(v.to_string().as_bytes())?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        debug!("serialize f32: {}", v);
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        debug!("serialize f64: {}", v);
        self.write_pending_key()?;
        self.writer.write_all(v.to_string().as_bytes())?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        debug!("serialize char: {}", v);
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        debug!("serialize &str: {}", v);

        if self.key {
            let upper = v.to_uppercase();
            if upper.contains(' ')
                || upper.contains('#')
                || upper.contains('\"')
                || upper.contains('\'')
            {
                return Err(Error::Syntax);
            }
            self.prefix.push(upper);
        } else if !v.is_empty() {
            self.write_pending_key()?;
            self.writer.write_all(b"\"")?;
            let mut b = [0; 4];
            for char in v.chars() {
                match char {
                    '\n' => {
                        self.writer.write_all(b"\\n")?;
                        continue;
                    }
                    '\\' | '"' | '$' => self.writer.write_all(b"\\")?,
                    _ => (),
                }

                self.writer.write_all(char.encode_utf8(&mut b).as_bytes())?;
            }
            self.writer.write_all(b"\"")?;
        }
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        debug!("serialize bytes: {:?}", v);
        self.serialize_str(&String::from_utf8(v.to_vec()).map_err(|_| Error::Syntax)?)
    }

    fn serialize_none(self) -> Result<()> {
        debug!("serialize none");
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::ser::Serialize,
    {
        debug!("serialize some");
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        debug!("serialize unit");
        self.write_pending_key()?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        debug!("serialize unit struct: {}", _name);
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        debug!("serialize unit variant: {}", variant);
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + serde::ser::Serialize,
    {
        debug!("serialize newtype struct: {}", _name);
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + serde::ser::Serialize,
    {
        debug!("serialize newtype struct variant: {}", variant);
        let SeqState::NotSeq = self.seq_state else {
            return value.serialize(&mut *self);
        };

        self.key = true;
        variant.serialize(&mut *self)?;
        self.key = false;
        self.writer.write_all(b"=")?;
        value.serialize(&mut *self)?;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        debug!("serialize sequence");
        let SeqState::NotSeq = self.seq_state else {
            return Err(Error::UnsupportedStructureInSeq);
        };
        self.seq_state = SeqState::First;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        debug!("serialize tuple");
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        debug!("serialize tuple struct");
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        debug!("serialize tuple variant");
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        debug!("serialize map");
        let SeqState::NotSeq = self.seq_state else {
            return Err(Error::UnsupportedStructureInSeq);
        };
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        debug!("serialize struct: {}", _name);
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        debug!("serialize struct variant: {}/{}", _name, _variant);
        self.serialize_map(Some(len))
    }
}

impl<W> serde::ser::SerializeSeq for &mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::ser::Serialize,
    {
        debug!("serializing sequence element");
        match self.seq_state {
            SeqState::First => self.seq_state = SeqState::Rest,
            SeqState::Rest => {
                self.writer.write_all(b",")?;
            }
            SeqState::NotSeq => unreachable!(),
        }
        let r = value.serialize(&mut **self);
        r
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing sequence element");
        self.seq_state = SeqState::NotSeq;
        Ok(())
    }
}

impl<W> serde::ser::SerializeTuple for &mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::ser::Serialize,
    {
        debug!("serialize tuple element");
        match self.seq_state {
            SeqState::First => self.seq_state = SeqState::Rest,
            SeqState::Rest => {
                self.writer.write_all(b",")?;
            }
            SeqState::NotSeq => unreachable!(),
        }

        let r = value.serialize(&mut **self);
        r
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing tuple element");
        self.seq_state = SeqState::NotSeq;
        Ok(())
    }
}

impl<W> serde::ser::SerializeTupleStruct for &mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::ser::Serialize,
    {
        debug!("serialize tuple struct field");
        match self.seq_state {
            SeqState::First => self.seq_state = SeqState::Rest,
            SeqState::Rest => {
                self.writer.write_all(b",")?;
            }
            SeqState::NotSeq => unreachable!(),
        }

        let r = value.serialize(&mut **self);
        r
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing tuple struct field");
        self.seq_state = SeqState::NotSeq;
        Ok(())
    }
}

impl<W> serde::ser::SerializeTupleVariant for &mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::ser::Serialize,
    {
        debug!("serialize tuple variant field");
        match self.seq_state {
            SeqState::First => self.seq_state = SeqState::Rest,
            SeqState::Rest => {
                self.writer.write_all(b",")?;
            }
            SeqState::NotSeq => unreachable!(),
        }
        let r = value.serialize(&mut **self);
        r
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing tuple variant field");
        self.seq_state = SeqState::NotSeq;
        Ok(())
    }
}

impl<W> serde::ser::SerializeMap for &mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + serde::ser::Serialize,
    {
        debug!("serialize map key");
        serialize_map_struct_key(self, key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::ser::Serialize,
    {
        debug!("serialize map value");
        serialize_map_struct_value(self, value)
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing map");
        Ok(())
    }
}

impl<W> serde::ser::SerializeStruct for &mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + serde::ser::Serialize,
    {
        debug!("serializing struct field");

        serialize_field(self, key, value)
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing struct field");
        Ok(())
    }
}

impl<W> serde::ser::SerializeStructVariant for &mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + serde::ser::Serialize,
    {
        debug!("serializing struct variant field");

        serialize_field(self, key, value)
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing struct variant");
        Ok(())
    }
}

fn serialize_field<W: std::io::Write, T>(
    ser: &'_ mut Serializer<W>,
    key: &'static str,
    value: &T,
) -> Result<()>
where
    T: ?Sized + serde::ser::Serialize,
{
    serialize_map_struct_key(ser, key)?;
    serialize_map_struct_value::<W, T>(ser, value)?;
    Ok(())
}

fn serialize_map_struct_key<W: std::io::Write, T>(ser: &'_ mut Serializer<W>, key: &T) -> Result<()>
where
    T: ?Sized + serde::ser::Serialize,
{
    let SeqState::NotSeq = ser.seq_state else {
        return Err(Error::UnsupportedStructureInSeq);
    };

    ser.key = true;
    key.serialize(&mut *ser)?;
    ser.key = false;
    ser.pending_key = true;
    Ok(())
}

fn serialize_map_struct_value<W: std::io::Write, T>(
    ser: &'_ mut Serializer<W>,
    value: &T,
) -> Result<()>
where
    T: ?Sized + serde::ser::Serialize,
{
    let SeqState::NotSeq = ser.seq_state else {
        return Err(Error::UnsupportedStructureInSeq);
    };

    value.serialize(&mut *ser)?;
    // ser.writer += "\n";

    ser.prefix.pop();
    ser.pending_key = false;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, io::Cursor};

    use super::{to_file, to_string, to_writer};
    use crate::{Value, from_str};

    #[test]
    fn serialize_to_string_value() {
        //* Given
        // TODO: Review this test case, the key is "hello" but the expected is "HELLO"
        let env = Value::from_iter([("hello", "WORLD")]);

        //* When
        let output = to_string(&env).expect("Failed to serialize to string");

        //* Then
        let expected = "HELLO=\"WORLD\"";
        assert_eq!(expected, &output);

        // Assert the deserialized value is equal to the original value
        let deserialized = from_str::<Value>(&output).expect("Failed to deserialize to value");
        assert_eq!(deserialized, env);
    }

    #[test]
    fn serialize_to_string_struct() {
        //* Given
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct StructTestNested {
            c: u8,
        }

        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct StructTest {
            a: u8,
            b: StructTestNested,
        }

        let env = StructTest {
            a: 1,
            b: StructTestNested { c: 2 },
        };

        //* When
        let output = to_string(&env).expect("Failed to serialize to string");

        //* Then
        assert_eq!("A=1\nB_C=2", output);
    }

    #[test]
    fn serialize_to_string_struct_field_after_nested() {
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Inner {
            x: u8,
        }

        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct Outer {
            a: u8,
            inner: Inner,
            b: u8,
        }

        let env = Outer {
            a: 1,
            inner: Inner { x: 2 },
            b: 3,
        };

        let output = to_string(&env).expect("Failed to serialize to string");

        assert_eq!("A=1\nINNER_X=2\nB=3", output);
    }

    #[test]
    fn serialize_to_string_deeply_nested_struct() {
        #[derive(Debug, PartialEq, serde::Serialize)]
        struct Level2 {
            z: u8,
        }

        #[derive(Debug, PartialEq, serde::Serialize)]
        struct Level1 {
            y: u8,
            level2: Level2,
            y2: u8,
        }

        #[derive(Debug, PartialEq, serde::Serialize)]
        struct Root {
            a: u8,
            level1: Level1,
            b: u8,
        }

        let env = Root {
            a: 1,
            level1: Level1 {
                y: 2,
                level2: Level2 { z: 3 },
                y2: 4,
            },
            b: 5,
        };

        let output = to_string(&env).expect("Failed to serialize to string");

        assert_eq!(
            "A=1\nLEVEL1_Y=2\nLEVEL1_LEVEL2_Z=3\nLEVEL1_Y2=4\nB=5",
            output
        );
    }

    #[test]
    fn serialize_to_string_sequence() {
        //* Given

        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct SeqTest {
            a: Vec<String>,
            b: String,
        }

        let env = SeqTest {
            a: vec!["HELLO".into(), "WORLD".into()],
            b: "control value".into(),
        };

        //* When
        let output = to_string(&env).expect("Failed to serialize to string");

        //* Then
        let expected = "A=\"HELLO\",\"WORLD\"\nB=\"control value\"";
        assert_eq!(expected, &output);

        // Assert the deserialized value is equal to the original value
        let deserialized = from_str::<SeqTest>(&output).expect("Failed to deserialize to struct");
        assert_eq!(deserialized, env);
    }

    #[test]
    fn serialize_to_string_enum() {
        //* Given
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        enum EnumTestEnum {
            HELLO,
            WORLD,
        }

        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct EnumTest {
            a: EnumTestEnum,
        }

        let env = EnumTest {
            a: EnumTestEnum::HELLO,
        };

        //* When
        let output = to_string(&env).expect("Failed to serialize to string");

        //* Then
        let expected = "A=\"HELLO\"";
        assert_eq!(expected, &output);

        // Assert the deserialized value is equal to the original value
        let deserialized = from_str::<EnumTest>(&output).expect("Failed to deserialize to struct");
        assert_eq!(deserialized, env);
    }

    #[test]
    fn serialize_to_string_numbers() {
        //* Given
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct NumberTest {
            u8: u8,
            u16: u16,
            u32: u32,
            u64: u64,
            i8: i8,
            i16: i16,
            i32: i32,
            i64: i64,
            f32: f32,
            f64: f64,
            usize: usize,
        }

        let env = NumberTest {
            u8: 255,
            u16: 65535,
            u32: 4294967295,
            u64: 18446744073709551615,
            i8: -128,
            i16: -32768,
            i32: -2147483648,
            i64: -9223372036854775808,
            f32: -3.5,
            f64: 3.5,
            usize: 18446744073709551615,
        };

        //* When
        let output = to_string(&env).expect("Failed to serialize to string");

        //* Then
        let expected = "U8=255\nU16=65535\nU32=4294967295\nU64=18446744073709551615\nI8=-128\nI16=-32768\nI32=-2147483648\nI64=-9223372036854775808\nF32=-3.5\nF64=3.5\nUSIZE=18446744073709551615";
        assert_eq!(expected, &output);

        // Assert the deserialized value is equal to the original value
        let deserialized =
            from_str::<NumberTest>(&output).expect("Failed to deserialize to struct");
        assert_eq!(deserialized, env);
    }

    #[test]
    fn serialize_to_string_bool() {
        //* Given
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct BoolTest {
            a: bool,
            b: bool,
        }

        let env = BoolTest { a: true, b: false };

        //* When
        let output = to_string(&env).expect("Failed to serialize to string");

        //* Then
        let expected = "A=true\nB=false";
        assert_eq!(expected, &output);

        // Assert the deserialized value is equal to the original value
        let deserialized = from_str::<BoolTest>(&output).expect("Failed to deserialize to struct");
        assert_eq!(deserialized, env);
    }

    #[test]
    fn serialize_to_string_option() {
        //* Given
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct OptionTest {
            a: Option<String>,
            b: Option<String>,
        }

        let env = OptionTest {
            a: Some("HELLO".into()),
            b: None,
        };

        //* When
        let output = to_string(&env).expect("Failed to serialize to string");

        //* Then
        let expected = "A=\"HELLO\"\nB=";
        assert_eq!(expected, &output);

        // Assert the deserialized value is equal to the original value
        // NOTE: envy deserializes "" with Some()
        let deserialized =
            from_str::<OptionTest>(&output).expect("Failed to deserialize to struct");
        assert_eq!(deserialized.a, env.a);
    }

    #[test]
    fn serialize_to_string_hashmap() {
        //* Given
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct MapTest {
            #[serde(flatten)]
            inner: HashMap<String, String>,
        }

        let mut env = MapTest {
            inner: HashMap::new(),
        };
        env.inner.insert("hello".into(), "WORLD".into());

        //* When
        let output = to_string(&env).expect("Failed to serialize to string");

        //* Then
        let expected = "HELLO=\"WORLD\"";
        assert_eq!(expected, &output);

        // Assert the deserialized value is equal to the original value
        let deserialized = from_str::<MapTest>(&output).expect("Failed to deserialize to struct");
        assert_eq!(deserialized, env);
    }

    #[test]
    fn serialize_to_string_nested_hashmap() {
        //* Given
        #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        struct NestedMapTest {
            inner: HashMap<String, String>,
        }

        let env = NestedMapTest {
            inner: HashMap::from([("HELLO".into(), "WORLD".into())]),
        };

        //* When
        let output = to_string(&env).expect("Failed to serialize to string");

        //* Then
        let expected = "INNER_HELLO=\"WORLD\"";
        assert_eq!(expected, &output);
    }

    #[test]
    fn serialize_to_writer() {
        //* Given
        let env = Value::from_iter([("HELLO", "WORLD")]);

        let mut writer = Cursor::new(Vec::new());

        //* When
        to_writer(&mut writer, &env).expect("Failed to serialize to writer");

        //* Then
        let expected_output = "HELLO=\"WORLD\"";
        let output = String::from_utf8(writer.into_inner()).expect("Invalid UTF-8 sequence");
        assert_eq!(expected_output, output);
    }

    #[test]
    fn serialize_to_file() {
        //* Given
        let env = Value::from_iter([("HELLO", "WORLD")]);

        let file = tempfile::NamedTempFile::new_in(std::env::temp_dir())
            .expect("Failed to create temp file");

        //* When
        to_file(&file.path(), &env).expect("Failed to serialize to file");

        //* Then
        let expected_output = "HELLO=\"WORLD\"";
        let output = std::fs::read_to_string(file.path()).expect("Failed to read file");
        assert_eq!(expected_output, output);
    }

    #[test]
    fn serialize_and_escape() {
        let mut env = Value::new();
        env.extend(
            vec![
                ("KEY", r"spaced value"),
                ("KEY2", r"value containing a $sign"),
                ("KEY3", r#"value containing a "quoted" value"#),
                ("KEY4", r"complex $\val'ue"),
                ("KEY5", r#"'"another\ complex value"#),
                ("KEY6", "value"),
                ("KEY7", "line 1\nline 2"),
                ("KEY8", "{\"hello\":\"world\"}"),
                (
                    "KEY9",
                    "-----BEGIN PRIVATE KEY-----
-----END PRIVATE KEY-----",
                ),
                ("KEY10", "${KEY}"),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<(String, String)>>(),
        );

        let output = to_string(&env).unwrap();

        let parsed: Value = from_str(&output).unwrap();

        for (key, value) in env.iter() {
            assert_eq!(value, parsed.get(&key.to_ascii_lowercase()).unwrap());
        }
    }
}
