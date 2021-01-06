%start BoolExpr
%avoid_insert "number" 
%avoid_insert "true" 
%avoid_insert "false"
%avoid_insert "identifier"
%%
BoolExpr -> Result<BoolExpr, ()>:
      BoolExpr 'eq' OrTerm { Ok(BoolExpr::EqBool($span, Box::new($1?), Box::new($3?))) }
    | OrTerm { $1 }
    ;

OrTerm -> Result<BoolExpr, ()>:
      OrTerm 'or' AndTerm {
        Ok(BoolExpr::Or($span, Box::new($1?), Box::new($3?)))
      }
    | AndTerm { $1 }
    ;

AndTerm -> Result<BoolExpr, ()>:
      AndTerm 'and' RootFactor {
        Ok(BoolExpr::And($span, Box::new($1?), Box::new($3?)))
      }
    | RootFactor { $1 }
    ;

RootFactor -> Result<BoolExpr, ()>:
      '(' BoolExpr ')' { $2 }
    | 'true' { Ok(BoolExpr::Const($span, true)) }
    | 'false' { Ok(BoolExpr::Const($span, false)) }
    | 'identifier' { Ok(BoolExpr::ReadBooleanInput($span, $lexer.span_str($span).to_string())) }
    | 'not' RootFactor { Ok(BoolExpr::Not($span, Box::new($2?))) }
    | Value '==' Value { Ok(BoolExpr::Equal($span, $1?, $3?)) }
    | Value '!=' Value { Ok(BoolExpr::Not($span, Box::new(BoolExpr::Equal($span, $1?, $3?)))) }
    | Value '>=' Value { Ok(BoolExpr::MoreThanOrEq($span, $1?, $3?)) }
    | Value '<=' Value { Ok(BoolExpr::LessThanOrEq($span, $1?, $3?)) }
    | Value '>' Value { Ok(BoolExpr::MoreThan($span, $1?, $3?)) }
    | Value '<' Value { Ok(BoolExpr::LessThan($span, $1?, $3?)) }
    | Value 'between' Value 'and' Value { Ok(BoolExpr::Between($span, $3?, $1?, $5?)) }
    | 'plus/minus' Value ',' Value '==' Value   { 
        Ok(BoolExpr::EqualPlusOrMinus($span, $2?, $4?, $6?)) 
      }
    ;

Value -> Result<Value, ()>:
    Value '+' Term {
    Ok(Value::Add(Box::new($1?), Box::new($3?)))
    }
  | Value '-' Term {
    Ok(Value::Sub(Box::new($1?), Box::new($3?)))
    }
  | Term { $1 } 
  ;

Term -> Result<Value, ()>:
    Term '*' Factor {
      Ok(Value::Mul(Box::new($1?), Box::new($3?)))
    }
  | Term '/' Factor {
      Ok(Value::Div(Box::new($1?), Box::new($3?)))
    }
  | Factor { $1 } 
  ;

Factor -> Result<Value, ()>:
      'number' { 
        Ok(Value::Const($lexer.span_str($span).parse().map_err(|_x| {
          ()
        })?)) 
      }
    | '(' Value ')' { $2 }
    | 'sun_declination' '(' DT ')' {
        Ok(Value::NoonSunDeclinationAngle($3?))
      }
    | 'hour_angle_sunrise' '(' LOC ',' DT ')' {
        Ok(Value::HourAngleSunrise($3?, $5?))
      }
    | 'hours_of_daylight' '(' LOC ',' DT ')' {
        Ok(Value::HoursOfDaylight($3?, $5?))
      }
    | 'hour_of_sunrise' '(' LOC ',' DT  ')' {
        Ok(Value::HourOfSunrise($3?, $5?))
      }
    | 'hour_of_sunset' '(' LOC ',' DT ')' {
        Ok(Value::HourOfSunrise($3?, $5?))
      }
    | 'offset_for_long' '(' LOC ')' {
        Ok(Value::HourOffset($3?))
      }
    | 'minute_of_hour' '(' DT ')' {
        Ok(Value::MinuteOfHour($3?))
      }
    | 'hour_of_day' '(' DT ')' {
        Ok(Value::HourOfDay($3?))
      }
    | 'week_day' '(' DT ')' {
        Ok(Value::WeekDayFromMonday($3?))
      }
    | 'year' '(' DT ')' {
        Ok(Value::Year($3?))
      }
    | 'month_of_year' '(' DT ')' {
        Ok(Value::MonthOfYear($3?))
      }
    | 'day_of_month' '(' DT ')' {
        Ok(Value::DayOfMonth($3?))
      }
    | 'day_of_year' '(' DT ')' {
        Ok(Value::DayOfYear($3?))
      }
    | 'read' '(' Identifier ',' Unit ')' {
        Ok(Value::ReadInput($3, $5?))
      }
    | 'lerp' '(' Value ',' Value ',' Value ')' {
        Ok(Value::Lerp(Box::new($3?), Box::new($5?), Box::new($7?)))
      }
    ;

Identifier -> String:
  'identifier' { $lexer.span_str($span).to_string() }
  ;

Unit -> Result<Unit, ()>:
    'degC' { Ok(Unit::DegC) }
  | 'bool' { Ok(Unit::Boolean) }
  | 'kpa' { Ok(Unit::KPa) }
  ;

DegNS -> Result<f64, ()>:
    'number' 'degN' {
      let num_span = $1.map_err(|_x| ())?.span();
      let num_str = $lexer.span_str(num_span);
      let num: f64 = num_str.parse().map_err(|_x| ())?;
      Ok(num) 
      }
  | 'number' 'degS' {
      let num_span = $1.map_err(|_x| ())?.span();
      let num_str = $lexer.span_str(num_span);
      let num: f64 = num_str.parse().map_err(|_x| ())?;
      Ok(-num) 
    }
  ;

DegEW -> Result<f64, ()>:
    'number' 'degE' {
      let num_span = $1.map_err(|_x| ())?.span();
      let num_str = $lexer.span_str(num_span);
      let num: f64 = num_str.parse().map_err(|_x| ())?;
      Ok(num) 
    }

  | 'number' 'degW' {
      let num_span = $1.map_err(|_x| ())?.span();
      let num_str = $lexer.span_str(num_span);
      let num: f64 = num_str.parse().map_err(|_x| ())?;
      Ok(-num) 
    }
  ;

LOC -> Result<LocationValue, ()>: 
    'here' { Ok(LocationValue::Here)  }
  | DegNS DegEW { 
      Ok(LocationValue::LatLong($1?, $2?)) 
  }
  ; 

DT -> Result<DateTimeValue, ()>: 
    'now' { Ok(DateTimeValue::Now) }
  | 'date' { 
        Ok(DateTimeValue::SpecificDate(
          $lexer.span_str($span).parse().map_err(|_x| ())?
          )) 
        }
  | 'date_time_z' { 
        Ok(DateTimeValue::SpecificDTZ(
          $lexer.span_str($span).parse().map_err(|_x| ())?
          )) 
        }
  | 'date_time' { 
        Ok(DateTimeValue::SpecificDT(
          $lexer.span_str($span).parse().map_err(|_x| ())?
          )) 
        }
  ;
  

Unmatched -> ():
      "UNMATCHED" { }
    ;
%%
use lrpar::Span;
use serde_derive::{Deserialize, Serialize};
use chrono::{NaiveDate, Local, NaiveDateTime, DateTime};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Unit {
    Boolean,
    DegC,
    KPa,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LocationValue {
  Here,
  LatLong(f64, f64)
}

#[derive(Clone, PartialEq, Debug)]
pub enum DateTimeValue {
  Now,
  SpecificDate(NaiveDate), // use local timezone of server
  SpecificDT(NaiveDateTime), // use local timezone of server
  SpecificDTZ(DateTime<Local>),
}

/// A source of f64 values, usable in expressions
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    // Some constant
    Const(f64),

    // angle of the sun (declination at noon, in radians)
    NoonSunDeclinationAngle(DateTimeValue),

    // hour-angle of sun at sunrise at a given lat and doy
    HourAngleSunrise(LocationValue, DateTimeValue),

    // How many hours of daylight are in day-of-year at latitude
    HoursOfDaylight(LocationValue, DateTimeValue),

    // Give local time of sunrise and sunset in local time hours
    HourOfSunrise(LocationValue, DateTimeValue),

    HourOfSunset(LocationValue, DateTimeValue),

    // Hour-offset (negative for west, positive for east) of a given longnitude
    HourOffset(LocationValue),

    // fractional minutes since start of this hour
    MinuteOfHour(DateTimeValue),

    // hour of day since midnight of this day
    HourOfDay(DateTimeValue),

    // Day of year, with fractional
    DayOfYear(DateTimeValue),

    // Mon=1, ..., Sun=7
    WeekDayFromMonday(DateTimeValue),

    // 2018, 2019...
    Year(DateTimeValue),

    // 1=Jan, 2=Feb
    MonthOfYear(DateTimeValue),

    // 1, 2, ... 30, 31
    DayOfMonth(DateTimeValue),

    // Current value of an input
    ReadInput(String, Unit),

    // linear interpolation  A * (1 - t) + B * t
    // where:
    //         A           t∈0..1      B
    Lerp(Box<Value>, Box<Value>, Box<Value>),

    // Transform y = Ax + b
    // where:
    //           A           x           b
    Linear(Box<Value>, Box<Value>, Box<Value>),

    // y = x + y
    Add(Box<Value>, Box<Value>),
    // y = x - y
    Sub(Box<Value>, Box<Value>),
    // y = x * y
    Mul(Box<Value>, Box<Value>),
    // y = x / y
    Div(Box<Value>, Box<Value>),

    // y = 1/x, x != 0
    Inverse(Box<Value>),

    // remove any floating point values (round-to-zero)
    Trunc(Box<Value>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum BoolExpr {
    Equal(Span, Value, Value),
    EqualPlusOrMinus(Span, Value, Value, Value),
    MoreThanOrEq(Span, Value, Value),
    LessThanOrEq(Span, Value, Value),
    MoreThan(Span, Value, Value),
    LessThan(Span, Value, Value),
    Between(Span, Value, Value, Value),
    Const(Span, bool),
    EqBool(Span, Box<BoolExpr>, Box<BoolExpr>),
    And(Span, Box<BoolExpr>, Box<BoolExpr>),
    Or(Span, Box<BoolExpr>, Box<BoolExpr>),
    Xor(Span, Box<BoolExpr>, Box<BoolExpr>),
    Not(Span, Box<BoolExpr>),
    ReadBooleanInput(Span, String),
}
