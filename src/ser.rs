use crate::error::{Error, Result};
#[cfg(feature = "debug")]
use log::debug;
use serde::{Serialize, ser};
use std::{fs::write, path::Path};

#[cfg(not(feature = "debug"))]
macro_rules! debug {
    ($fmt:expr $(, $arg:expr)*) => {};
}

/// A serializer to transform Rust data into environment variables.
pub struct Serializer {
    output: String,
    base_prefix: String,
    prefix: String,
    key: bool,
    sequence: bool,
    prefix_before: String,
}

impl Serializer {
    fn new(prefix: Option<&str>) -> Self {
        Self {
            output: String::new(),
            base_prefix: prefix.unwrap_or("").to_uppercase(),
            prefix: "".into(),
            key: false,
            sequence: false,
            prefix_before: "".into(),
        }
    }

    pub(crate) fn strip_line_breaks(&mut self) {
        while self.output.ends_with('\n') {
            self.output = self.output[..self.output.len() - 1].into();
        }
    }
}

/// Serialize data into an environment variable string.
///
/// # Example
///
/// ```
/// use serde_envfile::{Error, Value, to_string};
///
/// fn to_string_example() -> Result<(), Error> {
///     let mut value = Value::new();
///     value.insert("KEY".into(), "VALUE".into());
///     
///     let value: String = to_string(&value)?;
///     println!("{}", value);
///
///     Ok(())
/// }
/// ```
pub fn to_string<T>(v: &T) -> Result<String>
where
    T: ser::Serialize,
{
    to_string_inner(None, v)
}

pub fn to_string_inner<T>(prefix: Option<&str>, v: &T) -> Result<String>
where
    T: ser::Serialize,
{
    let mut serializer = Serializer::new(prefix);
    v.serialize(&mut serializer)?;

    Ok(serializer.output)
}

/// Serialize data into an environment variable file.
///
/// # Example
///
/// ```
/// use std::path::PathBuf;
/// use serde_envfile::{Error, Value, to_file};
///
/// fn to_string_example() -> Result<(), Error> {
///     let mut value = Value::new();
///     value.insert("KEY".into(), "VALUE".into());
///     
///     to_file(&PathBuf::from(".env"), &value)?;
///
///     Ok(())
/// }
/// ```
pub fn to_file<T>(p: &Path, v: &T) -> Result<()>
where
    T: ser::Serialize,
{
    to_file_inner(None, p, v)
}

pub fn to_file_inner<T>(prefix: Option<&str>, p: &Path, v: &T) -> Result<()>
where
    T: ser::Serialize,
{
    write(p, to_string_inner(prefix, v)?).map_err(|e| Error::Message(e.to_string()))
}

impl ser::Serializer for &mut Serializer {
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
        self.output += if v { "true" } else { "false" };
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
        self.output += &v.to_string();
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
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        debug!("serialize f32: {}", v);
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        debug!("serialize f64: {}", v);
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        debug!("serialize char: {}", v);
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        debug!("serialize &str: {}", v);

        if self.key {
            let mut key = self.base_prefix.clone();
            if !self.prefix.is_empty() {
                self.prefix.push('_');
            }
            self.prefix += &v.to_uppercase();
            key += &self.prefix;
            if key.find(' ').is_some()
                || key.find('#').is_some()
                || key.find('\"').is_some()
                || key.find('\'').is_some()
            {
                return Err(Error::Syntax);
            }

            self.output += &key;
        } else if !v.is_empty() {
            self.output += "\"";
            self.output += v;
            self.output += "\"";
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
        T: ?Sized + ser::Serialize,
    {
        debug!("serialize some");
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        debug!("serialize unit");
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        debug!("serialize unit struct: {}", name);
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

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        debug!("serialize newtype struct: {}", name);
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
        T: ?Sized + ser::Serialize,
    {
        debug!("serialize newtype struct variant: {}", variant);
        if self.sequence {
            return value.serialize(&mut *self);
        }

        self.key = true;
        variant.serialize(&mut *self)?;
        self.key = false;
        self.output += "=";
        value.serialize(&mut *self)?;
        self.output += "\n";
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        debug!("serialize sequence");
        if self.sequence {
            return Err(Error::UnsupportedStructureInSeq);
        }
        self.sequence = true;
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
        if self.sequence {
            return Err(Error::UnsupportedStructureInSeq);
        }
        Ok(self)
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        debug!("serialize struct: {}", name);
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        debug!("serialize struct variant: {}/{}", name, variant);
        self.serialize_map(Some(len))
    }
}

impl ser::SerializeSeq for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        debug!("serializing sequence element");
        let r = value.serialize(&mut **self);
        self.output += ",";
        r
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing sequence element");
        self.output.pop();
        self.sequence = false;
        self.strip_line_breaks();
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        debug!("serialize tuple element");
        let r = value.serialize(&mut **self);
        self.output += ",";
        r
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing tuple element");
        self.output.pop();
        self.sequence = false;
        self.strip_line_breaks();
        Ok(())
    }
}

impl ser::SerializeTupleStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        debug!("serialize tuple struct field");
        let r = value.serialize(&mut **self);
        self.output += ",";
        r
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing tuple struct field");
        self.output.pop();
        self.sequence = false;
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        debug!("serialize tuple variant field");
        let r = value.serialize(&mut **self);
        self.output += ",";
        r
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing tuple variant field");
        self.output.pop();
        self.sequence = false;
        Ok(())
    }
}

impl ser::SerializeMap for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        debug!("serialize map key");
        serialize_map_struct_key(self, key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        debug!("serialize map value");
        serialize_map_struct_value(self, value)
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing map");
        self.strip_line_breaks();
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        debug!("serializing struct field");

        serialize_field(self, key, value)
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing struct field");
        self.strip_line_breaks();
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        debug!("serializing struct variant field");

        serialize_field(self, key, value)
    }

    fn end(self) -> Result<()> {
        debug!("ended serializing struct variant");
        self.strip_line_breaks();
        Ok(())
    }
}

fn serialize_field<T>(ser: &'_ mut &'_ mut Serializer, key: &'static str, value: &T) -> Result<()>
where
    T: ?Sized + ser::Serialize,
{
    serialize_map_struct_key(ser, key)?;
    serialize_map_struct_value::<T>(ser, value)?;
    Ok(())
}

fn serialize_map_struct_key<T>(ser: &'_ mut &'_ mut Serializer, key: &T) -> Result<()>
where
    T: ?Sized + ser::Serialize,
{
    if ser.sequence {
        return Err(Error::UnsupportedStructureInSeq);
    }

    ser.prefix_before = ser.prefix.clone();

    let prefix = format!("{}{}", ser.prefix, '=');
    if ser.output.ends_with(&prefix) {
        ser.output = ser.output[..ser.output.len() - prefix.len()].into();
    }

    ser.key = true;
    key.serialize(&mut **ser)?;
    ser.key = false;
    Ok(())
}

fn serialize_map_struct_value<T>(ser: &'_ mut &'_ mut Serializer, value: &T) -> Result<()>
where
    T: ?Sized + ser::Serialize,
{
    if ser.sequence {
        return Err(Error::UnsupportedStructureInSeq);
    }

    ser.output += "=";
    value.serialize(&mut **ser)?;
    ser.output += "\n";

    ser.prefix = ser.prefix_before.clone();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::fs::read_to_string;
    use tempfile::NamedTempFile;

    use crate::{Value, from_str};

    #[test]
    fn to_string_test() {
        let mut env = Value::new();
        env.insert("HELLO".into(), "WORLD".into());

        let s = to_string(&env).unwrap();

        assert_eq!("HELLO=\"WORLD\"", s);
    }

    #[test]
    fn to_file_test() {
        let mut env = Value::new();
        env.insert("HELLO".into(), "WORLD".into());

        let file = NamedTempFile::new().unwrap();
        to_file(file.path(), &env).unwrap();
        let s = read_to_string(file.path()).unwrap();

        assert_eq!("HELLO=\"WORLD\"", s);
    }

    #[derive(Debug, Serialize)]
    struct StructTestNested {
        c: u8,
    }

    #[derive(Debug, Serialize)]
    struct StructTest {
        a: u8,
        b: StructTestNested,
    }

    #[test]
    fn struct_test() {
        let env = StructTest {
            a: 1,
            b: StructTestNested { c: 2 },
        };

        let s = to_string(&env).unwrap();

        assert_eq!("A=1\nB_C=2", s);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct SeqTest {
        a: Vec<String>,
        b: String,
    }

    #[test]
    fn seq_test() {
        let env = SeqTest {
            a: vec!["HELLO".into(), "WORLD".into()],
            b: "control value".into(),
        };

        let s = to_string(&env).unwrap();
        let expected = "A=\"HELLO\",\"WORLD\"\nB=\"control value\"";
        assert_eq!(expected, &s);
        assert_eq!(from_str::<SeqTest>(expected).unwrap(), env);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[allow(clippy::upper_case_acronyms)]
    enum EnumTestEnum {
        HELLO,
        WORLD,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct EnumTest {
        a: EnumTestEnum,
    }

    #[test]
    fn enum_test() {
        let env = EnumTest {
            a: EnumTestEnum::HELLO,
        };

        let s = to_string(&env).unwrap();
        let expected = "A=\"HELLO\"";
        assert_eq!(expected, &s);
        assert_eq!(from_str::<EnumTest>(&s).unwrap(), env);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
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

    #[test]
    fn number_test() {
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

        let s = to_string(&env).unwrap();
        let expected = "U8=255\nU16=65535\nU32=4294967295\nU64=18446744073709551615\nI8=-128\nI16=-32768\nI32=-2147483648\nI64=-9223372036854775808\nF32=-3.5\nF64=3.5\nUSIZE=18446744073709551615";
        assert_eq!(expected, &s);
        assert_eq!(from_str::<NumberTest>(&s).unwrap(), env);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct BoolTest {
        a: bool,
        b: bool,
    }

    #[test]
    fn bool_test() {
        let env = BoolTest { a: true, b: false };

        let s = to_string(&env).unwrap();
        let expected = "A=true\nB=false";
        assert_eq!(expected, &s);
        assert_eq!(from_str::<BoolTest>(&s).unwrap(), env);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct OptionTest {
        a: Option<String>,
        b: Option<String>,
    }

    #[test]
    fn option_test() {
        let env = OptionTest {
            a: Some("HELLO".into()),
            b: None,
        };

        let s = to_string(&env).unwrap();
        let expected = "A=\"HELLO\"\nB=";
        assert_eq!(expected, &s);
        assert_eq!(from_str::<OptionTest>(&s).unwrap().a, env.a);

        // envy deserializes "" with Some()
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct MapTest {
        #[serde(flatten)]
        inner: HashMap<String, String>,
    }

    #[test]
    fn map_test() {
        let mut env = MapTest {
            inner: HashMap::new(),
        };
        env.inner.insert("hello".into(), "WORLD".into());

        let s = to_string(&env).unwrap();
        let expected = "HELLO=\"WORLD\"";
        assert_eq!(expected, &s);
        assert_eq!(from_str::<MapTest>(&s).unwrap(), env);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct NestedMapTest {
        inner: HashMap<String, String>,
    }

    #[test]
    fn nested_map_test() {
        let mut env = NestedMapTest {
            inner: HashMap::new(),
        };
        env.inner.insert("hello".into(), "WORLD".into());

        let s = to_string(&env).unwrap();
        let expected = "INNER_HELLO=\"WORLD\"";
        assert_eq!(expected, &s);
    }
}
