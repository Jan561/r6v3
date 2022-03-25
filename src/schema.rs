table! {
    ts_members (user_id, removal_pending) {
        user_id -> BigInt,
        client_uuid -> Text,
        insertion_pending -> Bool,
        removal_pending -> Bool,
    }
}