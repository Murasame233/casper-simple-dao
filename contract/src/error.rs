use casper_types::ApiError;

#[repr(u16)]
pub enum CreateError {
    AlreadyJoin = 0,
}

impl From<CreateError> for ApiError {
    fn from(error: CreateError) -> Self {
        ApiError::User(error as u16)
    }
}

#[repr(u16)]
pub enum PlanError {
    NotOriginal = 0,
    AlreadyHaveProposal = 1,
    NoProposal = 2,
}

impl From<PlanError> for ApiError {
    fn from(error: PlanError) -> Self {
        ApiError::User(error as u16)
    }
}

#[repr(u16)]
pub enum Error {
    UnOpenEntry = 0,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::User(error as u16)
    }
}

#[repr(u16)]
pub enum OnlineError {
    UserHaveNoEnoughToken = 0,
    HaveUnFinishProposal = 1,
    InValidProposal = 2,
    NoPermission = 3,
    NoZero = 4,
    TooSmall = 5,
    AmountTooBig= 6,
}

impl From<OnlineError> for ApiError {
    fn from(error: OnlineError) -> Self {
        ApiError::User(error as u16)
    }
}
