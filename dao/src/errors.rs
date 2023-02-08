use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    NotInit = 0,
    MinDurationNotSatisfied = 1,
    CannotAddNegativeVote = 2,
    InvalidNonce = 3,
    AlreadyVoted = 4,
    InvalidProposalId = 5,
    NotEnoughPower = 6,
    TooEarlyToExecute = 7
}
