#[macro_export]
macro_rules! ok {
    ($($a:tt)*) => { println!("\x1b[32m✓\x1b[0m  {}", format!($($a)*)) }
}

#[macro_export]
macro_rules! arrow {
    ($($a:tt)*) => { println!("\x1b[36m→\x1b[0m  {}", format!($($a)*)) }
}

#[macro_export]
macro_rules! warn {
    ($($a:tt)*) => { println!("\x1b[33m~\x1b[0m  {}", format!($($a)*)) }
}

#[macro_export]
macro_rules! err {
    ($($a:tt)*) => { eprintln!("\x1b[31m✗\x1b[0m  {}", format!($($a)*)) }
}

#[macro_export]
macro_rules! head {
    ($($a:tt)*) => { println!("\x1b[1m==>\x1b[0m {}", format!($($a)*)) }
}
