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
    inputs (name) {
        name -> Text,
        name_as_entered -> Text,
        device_id -> Text,
        device_input_id -> Integer,
        unit -> Text,
        created_at -> Timestamp,
    }
}

table! {
    outputs (name) {
        name -> Text,
        name_as_entered -> Text,
        device_id -> Text,
        device_output_id -> Integer,
        unit -> Text,
        active_low -> Bool,
        automation_script -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(devices, inputs, outputs,);
