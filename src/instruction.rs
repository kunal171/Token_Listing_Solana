use solana_program::program_error::ProgramError;
use std::convert::TryInto;
// use borsh::{BorshDeserialize, BorshSerialize};

// use crate::error::EscrowError::InvalidInstruction;
// use std::io::Read;

#[derive(Debug, PartialEq)]
pub enum EscrowInstruction {
    CreatePlatformState {
        amount: u64,
    },

    CreateTokenstate, // List the new tokens for the trade

    ListToken {
        args: (u64, u64), //  NO of tokens to lsit and the prise of per token in the form of sol
    },

    Exchange {
        amount: u64, // No of tokens to buy
    },

    Cancel, // cancle the listing
}

impl EscrowInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => Self::CreatePlatformState {
                amount: Self::unpack_amount(rest)?,
            },
            1 => Self::CreateTokenstate,

            2 => Self::ListToken {
                args: Self::unpack_data(rest)?,
            },
            3 => Self::Exchange {
                amount: Self::unpack_amount(rest)?,
            },
            4 => Self::Cancel,

            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(amount)
    }

    fn unpack_data(input: &[u8]) -> Result<(u64, u64), ProgramError> {
        let amount1 = input
            .get(0..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        let amount2 = input
            .get(8..16)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok((amount1, amount2))
    }
}
