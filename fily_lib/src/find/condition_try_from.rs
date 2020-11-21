use std::{convert::TryFrom, fmt, error::Error};
use super::{Condition, SearchCriteria, SearchCriteriaParsingError};

// TODO: All of this

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionParsingError {
    SearchCriteriaParsingError(SearchCriteriaParsingError),
}

impl Error for ConditionParsingError {}

impl fmt::Display for ConditionParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<SearchCriteriaParsingError> for ConditionParsingError {
    fn from(error: SearchCriteriaParsingError) -> Self {
        Self::SearchCriteriaParsingError(error)
    }
}

impl TryFrom<&str> for Condition<SearchCriteria> {
    type Error = ConditionParsingError;

    fn try_from(_condition_str: &str) -> Result<Self, Self::Error> {
        todo!("no idea how to implement this");
    }
}
