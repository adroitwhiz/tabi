use serde_json::Value;
use std::{
    cmp::Ordering,
    convert::{Into, TryFrom},
};

#[derive(Debug, Clone)]
pub enum ScalarValue {
    Bool(bool),
    Num(f64),
    Text(Box<String>),
}

impl ScalarValue {
    pub fn compare(&self, other: &Self) -> Ordering {
        let mut n1: f64 = self.into();
        let mut n2: f64 = other.into();

        if n1 == 0f64 && self.is_whitespace() {
            n1 = f64::NAN;
        } else if n2 == 0f64 && other.is_whitespace() {
            n2 = f64::NAN
        }

        if n1.is_nan() || n2.is_nan() {
            // TODO: unnecessary copy?
            let s1: String = self.into();
            let s2: String = other.into();

            return s1.cmp(&s2);
        }

        if n1 == n2 {
            Ordering::Equal
        } else if n1 > n2 {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }

    fn is_whitespace(&self) -> bool {
        false // TODO
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

impl From<&ScalarValue> for bool {
    fn from(value: &ScalarValue) -> bool {
        match value {
            ScalarValue::Bool(v) => *v,
            ScalarValue::Num(v) => *v != 0.0 && *v != -0.0,
            ScalarValue::Text(v) => {
                let s = v.as_str();
                // TODO: Scratch's code doesn't check for the string "-0", but
                // tests seem to show that it is cast to false.
                !(s == "" || s == "0" || s == "-0" || s == "false")
            }
        }
    }
}

impl From<&ScalarValue> for f64 {
    fn from(value: &ScalarValue) -> f64 {
        match value {
            ScalarValue::Bool(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            ScalarValue::Num(v) => *v,
            // TODO: may not match JS semantics
            ScalarValue::Text(v) => v.parse::<f64>().unwrap_or(0.0),
        }
    }
}

impl From<&ScalarValue> for String {
    fn from(value: &ScalarValue) -> String {
        match value {
            ScalarValue::Bool(v) =>
                match v {
                    true => "true".to_string(),
                    false => "false".to_string()
                }
            ,
            // TODO: may not match JS semantics
            ScalarValue::Num(v) => v.to_string(),
            ScalarValue::Text(v) => *v.clone()
        }
    }
}
