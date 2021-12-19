use super::*;
use serde::ser::*;

impl<T: Serialize> WriteJs for T {
    fn write_js(&self, e: &mut JsEngine) -> Result<(), JsError> {
        self.serialize(JsEngineSerializer { engine: e, index: 0 })
    }
}

impl serde::ser::Error for JsError {
    fn custom<T>(msg: T) -> Self where T: std::fmt::Display {
        Self(msg.to_string())
    }
}

impl serde::de::Error for JsError {
    fn custom<T>(msg: T) -> Self where T: std::fmt::Display {
        Self(msg.to_string())
    }
}


struct JsEngineSerializer<'a> {
    engine: &'a mut JsEngine,
    index: u32,
}

impl<'a> Serializer for JsEngineSerializer<'a> {
    type Ok = ();
    type Error = JsError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.engine.push_boolean(v);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.engine.push_i32(v as i32);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.engine.push_i32(v as i32);
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.engine.push_i32(v);
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.engine.push_number(v as f64);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.engine.push_u32(v as u32);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.engine.push_u32(v as u32);
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.engine.push_u32(v as u32);
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.engine.push_number(v as f64);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.engine.push_number(v as f64);
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.engine.push_number(v);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut tmp = [0; 4];
        self.engine.push_string(v.encode_utf8(&mut tmp));
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.engine.push_string(v);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.engine.push_ext_buffer(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.engine.push_null();
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.engine.push_null();
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.engine.push_null();
        Ok(())
    }

    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> {
        self.engine.push_string(variant);
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok, Self::Error> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(self, _name: &'static str, _variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error> where T: Serialize {
        self.engine.push_object();
        value.serialize(JsEngineSerializer { engine: self.engine, index: 0 })?;
        self.engine.put_prop_string(-2, variant);
        Ok(())
    }

    fn serialize_seq(mut self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.engine.push_array();
        self.index = 0;
        Ok(self)
    }

    fn serialize_tuple(mut self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.engine.push_array();
        self.index = 0;
        Ok(self)
    }

    fn serialize_tuple_struct(mut self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.engine.push_array();
        self.index = 0;
        Ok(self)
    }

    fn serialize_tuple_variant(mut self, _name: &'static str, _variant_index: u32, variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.engine.push_object();
        self.engine.push_string(variant);
        self.engine.push_array();
        self.index = 0;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.engine.push_object();
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        self.engine.push_object();
        Ok(self)
    }

    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.engine.push_object();
        self.engine.push_string(variant);
        self.engine.push_object();
        Ok(self)
    }
}

impl<'a> SerializeSeq for JsEngineSerializer<'a> {
    type Ok = ();
    type Error = JsError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> where T: Serialize {
        value.serialize(JsEngineSerializer { engine: self.engine, index: 0 })?;
        self.engine.put_prop_index(-2, self.index);
        self.index += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeTuple for JsEngineSerializer<'a> {
    type Ok = ();
    type Error = JsError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> where T: Serialize {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeTupleVariant for JsEngineSerializer<'a> {
    type Ok = ();
    type Error = JsError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> where T: Serialize {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.engine.put_prop(-3);
        Ok(())
    }
}

impl<'a> SerializeTupleStruct for JsEngineSerializer<'a> {
    type Ok = ();
    type Error = JsError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> where T: Serialize {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeStruct for JsEngineSerializer<'a> {
    type Ok = ();
    type Error = JsError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> where T: Serialize {
        value.serialize(JsEngineSerializer { engine: self.engine, index: 0 })?;
        self.engine.put_prop_string(-2, key);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeStructVariant for JsEngineSerializer<'a> {
    type Ok = ();
    type Error = JsError;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> where T: Serialize {
        SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.engine.put_prop(-3);
        Ok(())
    }
}

impl<'a> SerializeMap for JsEngineSerializer<'a> {
    type Ok = ();
    type Error = JsError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> where T: Serialize {
        key.serialize(JsEngineSerializer { engine: self.engine, index: 0 })
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> where T: Serialize {
        value.serialize(JsEngineSerializer { engine: self.engine, index: 0 })?;
        self.engine.put_prop(-3);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use smart_default::SmartDefault;
    use serde::{Serialize, Deserialize};

    fn serialize<T: Serialize>(value: &T) -> String {
        let mut e = JsEngine::new();
        e.write(value).unwrap_or_else(|err| {
            panic!("{}", err);
        });
        e.put_global_string("value");
        e.eval(module_path!(), r#"var enc = Duktape.enc('jc', value, null, 2);"#);
        e.get_global_string("enc");
        e.get_string(-1).to_string()
    }

    fn test_serialize<T: Serialize>(value: &T) {
        let s = serialize(value);
        assert_eq!(serde_json::to_string_pretty(value).unwrap(), s);
    }

    #[derive(SmartDefault, Serialize, Deserialize)]
    struct TestStruct {
        #[default = "string value"]
        string_field: String,
        #[default = 'A']
        char_field: char,
        #[default = 1]
        i8_field: i8,
    }

    #[derive(SmartDefault, Serialize, Deserialize)]
    struct TestStruct2 {
        struct_field: TestStruct,
        #[default(_code = "('A', 'B', 'C')")]
        unit_char_field: (char, char, char),
        #[default(_code = "vec![1, 2 ,3]")]
        array_i8_field: Vec<i8>,
    }

    #[test]
    fn write_string() {
        let p = String::from("string value");
        test_serialize(&p);
    }

    #[test]
    fn write_bytes() {
        struct Bytes<'a>(&'a [u8]);

        impl<'a> Serialize for Bytes<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
                serializer.serialize_bytes(self.0)
            }
        }

        let p = Bytes("byte data".as_bytes());
        let ser = serialize(&p);
        assert_eq!(ser, r#"{"_buf":"627974652064617461"}"#);
        assert_eq!(p.0, "byte data".as_bytes());
    }

    #[test]
    fn write_struct() {
        let p = TestStruct2::default();
        test_serialize(&p);
    }

    #[test]
    fn write_option_none() {
        let p: Option<TestStruct2> = None;
        test_serialize(&p);
    }

    #[test]
    fn write_option_some() {
        let p: Option<TestStruct2> = Some(TestStruct2::default());
        test_serialize(&p);
    }

    #[test]
    fn write_enum_unit_variant() {
        #[derive(SmartDefault, Serialize, Deserialize)]
        enum TestEnum {
            #[default]
            Empty,
        }
        let p = TestEnum::default();
        test_serialize(&p);
    }

    #[test]
    fn write_enum_tuple_variant() {
        #[derive(SmartDefault, Serialize, Deserialize)]
        enum TestEnum {
            #[default]
            Tuple(
                #[default = "string val"]
                String,
                TestStruct,
                #[default(_code = "(1, 2, 12.5)")]
                (i8, u32, f64),
            ),
        }
        let p = TestEnum::default();
        test_serialize(&p);
    }

    #[test]
    fn write_enum_struct_variant() {
        #[derive(SmartDefault, Serialize, Deserialize)]
        enum TestEnum {
            #[default]
            Struct {
                #[default = "string val"]
                str_field: String,
                struct_field: TestStruct,
                #[default = 12.5]
                float_field: f64,
            },
        }
        let p = TestEnum::default();
        test_serialize(&p);
    }

    #[test]
    fn write_struct_variant() {
        #[derive(SmartDefault, Serialize, Deserialize)]
        struct TestStructVariant(
            #[default = "string val"]
            String,
            TestStruct,
            #[default = 12.5]
            f64
        );

        let p = TestStructVariant::default();
        test_serialize(&p);
    }

    #[test]
    fn write_unit_struct() {
        #[derive(Serialize, Deserialize)]
        struct TestUnitStruct;

        let p = TestUnitStruct;
        test_serialize(&p);
    }

    #[test]
    fn write_newtype_struct() {
        #[derive(Serialize, Deserialize)]
        struct TestNewtypeStruct(f64);

        let p = TestNewtypeStruct(15.9);
        test_serialize(&p);
    }

    #[test]
    fn write_newtype_variant() {
        #[derive(Serialize, Deserialize)]
        enum TestEnum {
            Newtype(f64)
        }

        let p = TestEnum::Newtype(15.9);
        test_serialize(&p);
    }

    #[test]
    fn write_map() {
        use std::collections::BTreeMap;
        let mut map: BTreeMap<&str, TestStruct> = BTreeMap::new();
        map.insert("key1", TestStruct::default());
        map.insert("key2", TestStruct::default());
        map.insert("key3", TestStruct::default());

        test_serialize(&map);
    }
}
