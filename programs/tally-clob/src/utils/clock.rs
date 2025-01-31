use solana_program::clock::UnixTimestamp;

pub fn current_timestamp() -> UnixTimestamp {
    use solana_program::clock::Clock;
    use solana_program::sysvar::Sysvar;
    Clock::get().unwrap().unix_timestamp
}