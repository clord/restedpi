// @generated automatically by Diesel CLI.

diesel::table! {
    devices (name) {
        name -> Text,
        model -> Text,
        notes -> Text,
        disabled -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    inputs (name) {
        name -> Text,
        device_id -> Text,
        device_input_id -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    outputs (name) {
        name -> Text,
        device_id -> Text,
        device_output_id -> Integer,
        active_low -> Bool,
        automation_script -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(devices, inputs, outputs,);
