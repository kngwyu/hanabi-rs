use error_chain_mini::ChainedError;
pub(crate) use error_chain_mini::{ErrorKind, ResultExt};
use {CardId, CardInfo};
#[derive(Clone, Debug, ErrorKind)]
pub enum CoreError {
    #[msg(short = "invalid card id", detailed = "{:?}", _0)]
    InvalidCard(CardId),
    #[msg(short = "invalid player id", detailed = "{:?}", _0)]
    InvalidPlayer(usize),
    #[msg(short = "incorrect player id", detailed = "{:?}", _0)]
    IncorrectInfo(CardInfo),
}

pub type Error = ChainedError<CoreError>;
