table! {
    movie_channels (id) {
        id -> Binary,
        uri -> Text,
        vc -> BigInt,
        bot_msg_channel_id -> BigInt,
        bot_msg -> BigInt,
        guild -> BigInt,
        creator -> BigInt,
        created_at -> Timestamp,
    }
}
