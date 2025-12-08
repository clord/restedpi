use crate::config::types::Unit;
use juniper::{GraphQLObject, GraphQLUnion};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct DimMessage {
    pub message: String,
}

#[derive(Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct DimBool {
    pub value: bool,
}

#[derive(Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct DimDegC {
    pub value: f64,
}
#[derive(Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct DimKPa {
    pub value: f64,
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
    pub fn from_degc(value: f64) -> Dimensioned {
        Dimensioned::DegC(DimDegC { value })
    }
    pub fn from_kpa(value: f64) -> Dimensioned {
        Dimensioned::KPa(DimKPa { value })
    }

    pub fn from_bool(value: bool) -> Dimensioned {
        Dimensioned::Boolean(DimBool { value })
    }

    pub fn value(&self) -> crate::error::Result<f64> {
        match self {
            Self::DegC(DimDegC { value }) => Ok(*value),
            Self::Boolean(DimBool { value }) => Ok(if *value { 1.0 } else { 0.0 }),
            Self::KPa(DimKPa { value }) => Ok(*value),
            Self::Error(DimMessage { message }) => {
                Err(crate::error::Error::UnitError(message.clone()))
            }
        }
    }

    pub fn unit(&self) -> crate::error::Result<Unit> {
        match self {
            Self::DegC(_) => Ok(Unit::DegC),
            Self::Boolean(_) => Ok(Unit::Boolean),
            Self::KPa(_) => Ok(Unit::KPa),
            Self::Error(DimMessage { message }) => {
                Err(crate::error::Error::UnitError(message.clone()))
            }
        }
    }

    pub fn new(unit: Unit, value: f64) -> Dimensioned {
        match unit {
            Unit::DegC => Dimensioned::DegC(DimDegC { value }),
            Unit::Boolean => Dimensioned::Boolean(DimBool { value: value > 0.0 }),
            Unit::KPa => Dimensioned::KPa(DimKPa { value }),
        }
    }
    pub fn is_unit(&self, unit: Unit) -> bool {
        matches!(
            (unit, self),
            (Unit::KPa, &Dimensioned::KPa(_))
                | (Unit::DegC, &Dimensioned::DegC(_))
                | (Unit::Boolean, &Dimensioned::Boolean(_))
        )
    }
}
