macro_rules! tri {
    ($res:expr, $log:expr $(,)?) => {
        if let Err(why) = $res {
            log::warn!("{}: {:?}.", $log, why);
        }
    };
}
