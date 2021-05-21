table! {
    devices (name) {
        name -> Text,
        name_as_entered -> Text,
        model -> Text,
        notes -> Text,
        disabled -> Bool,
        created_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::{Text, Timestamp, Integer};
    use crate::config::types::UnitMapping;
    inputs (name) {
        name -> Text,
        name_as_entered -> Text,
        device_id -> Text,
        device_input_id -> Integer,
        unit -> UnitMapping,
        created_at -> Timestamp,
    }
}

table! {
    use diesel::sql_types::{Nullable, Bool, Text, Timestamp, Integer};
    use crate::config::types::UnitMapping;
    outputs (name) {
        name -> Text,
        name_as_entered -> Text,
        device_id -> Text,
        device_output_id -> Integer,
        unit -> UnitMapping,
        active_low -> Bool,
        automation_script -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(devices, inputs, outputs,);
