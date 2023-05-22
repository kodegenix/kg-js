use super::*;
use serde::de::*;


impl<'de, T: Deserialize<'de>> ReadJs for T {
    fn read_js(e: &mut JsEngine, obj_index: i32) -> Result<Self, JsError> {
        Self::deserialize(JsEngineDeserializer::new(e, obj_index))
    }
}

pub struct JsEngineDeserializer<'a> {
    engine: &'a mut JsEngine,
    index: i32,
    len: usize,
}

impl <'a> JsEngineDeserializer<'a> {
    pub fn new(engine: &'a mut JsEngine, index: i32) -> Self {
        Self { engine, index, len: 0 }
    }
}

impl<'de, 'a> Deserializer<'de> for JsEngineDeserializer<'a> {
    type Error = JsError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        use super::DukType::*;

        match self.engine.get_type(self.index) {
            DUK_TYPE_UNDEFINED | DUK_TYPE_NULL => visitor.visit_none(),
            DUK_TYPE_BOOLEAN => visitor.visit_bool(self.engine.get_boolean(self.index)),
            DUK_TYPE_NUMBER => {
                let n = self.engine.get_number(self.index);
                if n.is_finite() && (n.trunc() - n).abs() < f64::EPSILON {
                    visitor.visit_i64(n as i64)
                } else {
                    visitor.visit_f64(n)
                }
            }
            DUK_TYPE_STRING => visitor.visit_str(self.engine.get_string(self.index)),
            DUK_TYPE_BUFFER => visitor.visit_bytes(self.engine.get_buffer(self.index)),
            DUK_TYPE_OBJECT => {
                if self.engine.is_array(self.index) {
                    let len = self.engine.get_length( self.index);
                    self.engine.enum_indices(self.index);
                    let res = visitor.visit_seq(JsEngineDeserializer { engine: self.engine, index: -1, len });
                    self.engine.pop();
                    res
                } else if self.engine.is_pure_object(self.index) {
                    self.engine.enum_keys(self.index);
                    let res = visitor.visit_map(JsEngineDeserializer { engine: self.engine, index: -1, len: 0 });
                    self.engine.pop();
                    res
                } else {
                    return Err(JsError(format!("Unimplemented javascript object type"))); //FIXME (jc)
                }
            }
            _ => return Err(JsError(format!("Unimplemented javascript object type"))) //FIXME (jc),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        use super::DukType::{DUK_TYPE_NULL, DUK_TYPE_UNDEFINED};

        match self.engine.get_type(self.index) {
            DUK_TYPE_UNDEFINED | DUK_TYPE_NULL => visitor.visit_none(),
            _ => visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, _len: usize, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_struct<V>(self, _name: &'static str, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_enum<V>(self, _name: &'static str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
        self.deserialize_any(visitor)
    }
}

impl<'de, 'a> MapAccess<'de> for JsEngineDeserializer<'a> {
    type Error = JsError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error> where K: DeserializeSeed<'de> {
        if self.engine.next(-1) {
            Ok(Some(seed.deserialize(JsEngineDeserializer { engine: self.engine, index: -2, len: 0 })?))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error> where V: DeserializeSeed<'de> {
        let res = seed.deserialize(JsEngineDeserializer { engine: self.engine, index: -1, len: 0 });
        self.engine.pop_n(2);
        res
    }

    fn next_entry_seed<K, V>(&mut self, kseed: K, vseed: V) -> Result<Option<(K::Value, V::Value)>, Self::Error> where K: DeserializeSeed<'de>, V: DeserializeSeed<'de> {
        if self.engine.next(-1) {
            let k = kseed.deserialize(JsEngineDeserializer { engine: self.engine, index: -2, len: 0 })?;
            let v = vseed.deserialize(JsEngineDeserializer { engine: self.engine, index: -1, len: 0 })?;
            self.engine.pop_n(2);
            Ok(Some((k, v)))
        } else {
            Ok(None)
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for JsEngineDeserializer<'a> {
    type Error = JsError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error> where T: DeserializeSeed<'de> {
        if self.engine.next(-1) {
            let v = seed.deserialize(JsEngineDeserializer { engine: self.engine, index: -1, len: 0 })?;
            self.engine.pop_n(2);
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use smart_default::SmartDefault;
    use serde::{Serialize, Deserialize};

    fn deserialize<'a, T: std::fmt::Debug + Serialize + Deserialize<'a> + Default>(value: &T) {
        let mut e = JsEngine::new();
        e.write(value).unwrap_or_else(|err| {
            panic!("{}", err);
        });
        e.put_global_string("value");
        e.get_global_string("value");
        let val: T = e.read_top().unwrap_or_else(|err| {
            panic!("{}", err);
        });
        assert_eq!(format!("{:?}", value), format!("{:?}", val));
    }

    fn deserialize_expr<'a, T: std::fmt::Debug + Deserialize<'a>>(expr: &str) -> T {
        let mut e = JsEngine::new();
        e.eval(expr).unwrap();
        e.get_global_string("value");
        let val: T = e.read_top().unwrap_or_else(|err| {
            panic!("{}", err);
        });
        val
    }

    fn test_deserialize<'a, T: std::fmt::Debug + Serialize + Deserialize<'a> + Default>(value: &T) {
        deserialize(value);
    }

    #[derive(Debug, SmartDefault, Serialize, Deserialize)]
    struct TestStruct {
        #[default = "string value"]
        string_field: String,
        #[default = 'A']
        char_field: char,
        #[default = 1]
        i8_field: i8,
        #[default(_code = "vec![1.0,2.0,3.0,7.5]")]
        arr_field: Vec<f64>,
        optional1: Option<f64>,
        optional2: Option<f64>
    }

    #[test]
    fn read_struct() {
        let mut p = TestStruct::default();
        p.char_field = 'B';
        p.i8_field = 44;
        p.optional1 = Some(3.14);
        test_deserialize(&p);
    }

    #[test]
    fn deserialize_zero() {
        #[derive(Debug, Deserialize)]
        struct TestStruct {
            float_field: f64,
            int_field: i64,
        }

        //language=JavaScript
        let val: TestStruct = deserialize_expr(r#"
    value = {
        float_field: 0,
        int_field: 0
    };
"#);
        assert_eq!(val.float_field, 0.0);
        assert_eq!(val.int_field, 0);
    }

    #[test]
    fn deserialize_nan() {
        #[derive(Debug, Deserialize)]
        struct TestStruct {
            float_field: f64,
        }

        //language=JavaScript
        let val: TestStruct = deserialize_expr(r#"
    value = {
        float_field: NaN
    };
"#);
        assert!(val.float_field.is_nan());
    }
}
