use cosmwasm_std::{StdResult, StdError};
use crate::babylonian::Babylonian;

// range: [0, 2**112 - 1]
// resolution: 1 / 2**112
pub struct uq112x112 {
    pub _x: u128
}

// range: [0, 2**144 - 1]
// resolution: 1 / 2**112
pub struct uq144x112 {
    pub _x: u128
}

pub struct FixedPoint{
    RESOLUTION: u128,
    Q112: u128,
    Q224: u128
}
impl Default for FixedPoint{
    fn default() -> FixedPoint{
        let q112 = 1 << 112;
        let q224 = q112 << 112;
        FixedPoint{
            RESOLUTION: 112,
            Q112: q112,
            Q224: q224
        }
    }
}

impl FixedPoint{
    // encode a uint112 as a UQ112x112
    pub fn encode(self, x: u128) -> StdResult<uq112x112> {
        Ok(uq112x112{
            _x: x << self.RESOLUTION
        })
    }
    // encodes a uint144 as a UQ144x112
    pub fn encode144(self, x: u128) -> StdResult<uq144x112> {
        Ok(uq144x112{
            _x: x << self.RESOLUTION
        })
    }

    // divide a UQ112x112 by a uint112, returning a UQ112x112
    pub fn div(_self: uq112x112, x: u128)-> StdResult<uq112x112> {
        if x == 0 {
            return Err(StdError::GenericErr{
                msg:"Divide by zero".to_string()
            });
        }
        Ok(uq112x112{
            _x: _self._x / x
        })
    }

    // multiply a UQ112x112 by a uint, returning a UQ144x112
    // reverts on overflow
    pub fn mul( _self: uq112x112, y: u128) -> StdResult<uq144x112> {
        let z= _self._x * y;
        if y != 0 && z / y != _self._x {
            return Err(StdError::GenericErr{
                msg: "FixedPoint: MULTIPLICATION_OVERFLOW".to_string()
            });
        }
        Ok(uq144x112{
            _x: z
        })
    }

    // returns a UQ112x112 which represents the ratio of the numerator to the denominator
    // equivalent to encode(numerator).div(denominator)
    pub fn fraction(self, numerator: u128, denominator: u128) -> StdResult<uq112x112> {
        if denominator <=0 {
            return Err(StdError::GenericErr{
                msg: "FixedPoint: DIV_BY_ZERO".to_string()
            })
        }
        Ok(uq112x112{
            _x: (numerator << self.RESOLUTION) / denominator
        })
    }

    // decode a UQ112x112 into a uint112 by truncating after the radix point
    pub fn decode(self, _self: uq112x112) -> StdResult<u128> {
        Ok(_self._x >> self.RESOLUTION)
    }
    
    // decode a UQ144x112 into a uint144 by truncating after the radix point
    pub fn decode144(self, _self: uq144x112) -> StdResult<u128> {
        Ok(_self._x >> self.RESOLUTION)
    }

    // take the reciprocal of a UQ112x112
    pub fn reciprocal(self, _self: uq112x112) -> StdResult<uq112x112> {
        if _self._x == 0 {
            return Err(StdError::GenericErr{
                msg: "FixedPoint: ZERO_RECIPROCAL".to_string()
            });
        }
        Ok(uq112x112{
            _x: self.Q224 / _self._x
        })
    }    

    // square root of a UQ112x112
    pub fn sqrt(_self: uq112x112) -> StdResult<uq112x112> {
        Ok(uq112x112{
            _x: Babylonian::sqrt(_self._x) << 56
        })
    }    
}
