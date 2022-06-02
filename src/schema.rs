table! {
    movie_channels (id) {
        id -> Binary,
        uri -> Text,
        vc -> BigInt,
        bot_msg -> BigInt,
        creator -> BigInt,
        created_at -> Timestamp,
    }
}
