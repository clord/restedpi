table! {
    devices (device_id) {
        device_id -> Integer,
        model_type -> Text,
        model_config -> Text,
        name -> Text,
        notes -> Text,
        disabled -> Bool,
        created_at -> Timestamp,
    }
}

table! {
    inputs (input_id) {
        input_id -> Integer,
        name -> Text,
        device_id -> Integer,
        device_input_id -> Integer,
        unit -> Text,
        created_at -> Timestamp,
    }
}

table! {
    outputs (output_id) {
        output_id -> Integer,
        name -> Text,
        device_id -> Integer,
        device_output_id -> Integer,
        unit -> Text,
        active_low -> Bool,
        automation_script -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

joinable!(inputs -> devices (device_id));
joinable!(outputs -> devices (device_id));

allow_tables_to_appear_in_same_query!(
    devices,
    inputs,
    outputs,
);
