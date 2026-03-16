use rand::distr::Alphanumeric;

use rand::{ Rng, RngExt};
// use rand::distributions::Alphanumeric;
pub fn generate_random_string(length: usize) -> String {
    let rng = rand::rng();
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}