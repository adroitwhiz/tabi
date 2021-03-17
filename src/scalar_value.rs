use serde_json::Value;
use std::{cmp::Ordering, convert::TryFrom};

#[derive(Debug, Clone)]
pub enum ScalarValue {
    Bool(bool),
    Num(f64),
    Text(Box<String>),
}

impl ScalarValue {
    pub fn compare(&self, _other: &Self) -> Ordering {
        unimplemented!()
    }
}

impl TryFrom<&Value> for ScalarValue {
    type Error = &'static str;
    fn try_from(value: &Value) -> Result<ScalarValue, Self::Error> {
        match value {
            Value::Bool(v) => Ok(ScalarValue::Bool(*v)),
            Value::Number(v) => {
                if let Some(n) = v.as_f64() {
                    Ok(ScalarValue::Num(n))
                } else {
                    Err("Could not convert Serde Number to f64")
                }
            }
            Value::String(v) => Ok(ScalarValue::Text(Box::new(v.clone()))),
            _ => {
                println!("{:?}", value);
                Err("Could not convert Serde value to ScalarValue")
            }
        }
    }
}

impl From<ScalarValue> for bool {
    fn from(value: ScalarValue) -> bool {
        match value {
            ScalarValue::Bool(v) => v,
            ScalarValue::Num(v) => v != 0.0 && v != -0.0,
            ScalarValue::Text(v) => {
                let s = *v;
                // TODO: Scratch's code doesn't check for the string "-0", but
                // tests seem to show that it is cast to false.
                !(s == "" || s == "0" || s == "-0" || s == "false")
            }
        }
    }
}

impl From<ScalarValue> for f64 {
    fn from(value: ScalarValue) -> f64 {
        match value {
            ScalarValue::Bool(v) => {
                if v {
                    1.0
                } else {
                    0.0
                }
            }
            ScalarValue::Num(v) => v,
            ScalarValue::Text(v) => v.parse::<f64>().unwrap_or(0.0),
        }
    }
}
