/// To protect the pool from unfair advantages, a small amount of liquidity is always
/// minted and sent to a "dead" address when the pool is first created.
/// This ensures the first person to deposit doesn't receive excessive LP tokens.
pub const MINIMUM_LIQUIDITY: u64 = 1_000;
