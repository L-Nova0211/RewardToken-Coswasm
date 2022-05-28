use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Babylonian {
}

impl Babylonian{
    pub fn sqrt(y: u128) -> u128
    {
        let mut z = 0;
        if y > 3 {
            z = y;
            let mut x = y / 2 + 1;
            while x < z {
                z = x;
                x = (y / x + x) / 2;
            }
        } else if y != 0 {
            z = 1;
        }
        // else z = 0
        z
    }
}