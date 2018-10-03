table! {
    users (username) {
        username -> Text,
        color -> Text,
        password -> Text,
    }
}

table! {
    messages (id) {
        id -> BigInt,
        username -> Text,
        message -> Text,
    }
}
