use crate::config::types::Unit;
use juniper::{GraphQLObject, GraphQLUnion};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct DimMessage {
    message: String,
}

#[derive(Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct DimBool {
    value: bool,
}

#[derive(Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct DimDegC {
    value: f64,
}
#[derive(Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct DimKPa {
    value: f64,
}

#[derive(Serialize, Deserialize, GraphQLUnion, PartialEq, Clone, Debug)]
#[serde(tag = "dim")]
pub enum Dimensioned {
    Error(DimMessage),
    Boolean(DimBool),
    DegC(DimDegC),
    KPa(DimKPa),
}

impl Dimensioned {
    pub fn from_error(message: String) -> Dimensioned {
        Dimensioned::Error(DimMessage { message })
    }
    pub fn new(unit: Unit, value: f64) -> Dimensioned {
        match unit {
            Unit::DegC => Dimensioned::DegC(DimDegC { value }),
            Unit::Boolean => Dimensioned::Boolean(DimBool { value: value > 0.0 }),
            Unit::KPa => Dimensioned::KPa(DimKPa { value }),
        }
    }
}
