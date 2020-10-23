use std::convert::TryFrom;
use thiserror::Error;
use super::{Condition, SearchCriteria, SearchCriteriaParsingError};

// TODO: All of this

#[derive(Debug, Clone, PartialEq, Error)]
pub enum ConditionParsingError {
    #[error("Something went wrong parsing the SearchCriteria")]
    SearchCriteriaParsingError(SearchCriteriaParsingError),
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
