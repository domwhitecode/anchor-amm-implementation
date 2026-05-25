use anchor_lang::prelude::*;
use constant_product_curve::CurveError;

#[error_code]
pub enum AmmError {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Pool is locked, try again later")]
    PoolLocked,

    #[msg("Invalid deposit amount ")]
    InvalidAmount,

    #[msg("Slippage Exceeded")]
    SlippageExceeded,
    #[msg("Invalid Precision")]
    InvalidPrecision,
    #[msg("Overflow")]
    OverFlow,
    #[msg("Underflow")]
    UnderFlow,
    #[msg("Invalid Fee Amount")]
    InvalidFeeAmount,
    #[msg("Insufficient Balance")]
    InsufficientBalance,
    #[msg("ZeroBalance")]
    ZeroBalance,
    #[msg("Slippage Limit Exceeded")]
    SlippageLimitExceeded,
    #[msg("XY deposit/withdraw amounts from l failed")]
    XYCalculationFailed,
    #[msg("Failed to initialize the CPMM curve")]
    CPMMInitFail,
}

impl From<CurveError> for AmmError {
    fn from(error: CurveError) -> AmmError {
        match error {
            CurveError::InvalidPrecision => AmmError::InvalidPrecision,
            CurveError::Overflow => AmmError::OverFlow,
            CurveError::Underflow => AmmError::UnderFlow,
            CurveError::InvalidFeeAmount => AmmError::InvalidFeeAmount,
            CurveError::InsufficientBalance => AmmError::InsufficientBalance,
            CurveError::ZeroBalance => AmmError::ZeroBalance,
            CurveError::SlippageLimitExceeded => AmmError::SlippageLimitExceeded,
        }
    }
}
