use raydium_amm_v3::libraries::fixed_point_64;

// Utils method to convert price in different format
// Copied from: https://github.com/raydium-io/raydium-clmm/blob/master/client/src/instructions/utils.rs#L235-L276
pub fn multipler(decimals: u8) -> f64 {
    (10_i32).checked_pow(decimals.try_into().unwrap()).unwrap() as f64
}

pub fn from_x64_price(price: u128) -> f64 {
    price as f64 / fixed_point_64::Q64 as f64
}

pub fn sqrt_price_x64_to_price(price: u128, decimals_0: u8, decimals_1: u8) -> f64 {
    from_x64_price(price).powi(2) * multipler(decimals_0) / multipler(decimals_1)
}