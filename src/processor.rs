use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction::{create_account, transfer},
    sysvar::rent::Rent,
};

use crate::{
    instruction::EscrowInstruction,
    state::{ListerState, PlatformState, TokenState},
};
use std::str::FromStr;
pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        // Escrow instructions
        match instruction {
            EscrowInstruction::CreatePlatformState { amount } => {
                msg!("Instruction: Update platform accounts");
                Self::process_create_platform_state(accounts, amount, program_id)
            }
            EscrowInstruction::CreateTokenstate => {
                msg!("Instruction: Create Tokenstate accounts");
                Self::process_create_token_state(accounts, program_id)
            }
            EscrowInstruction::ListToken { args } => {
                msg!("Instruction: ListToken");
                Self::process_init_escrow(accounts, args, program_id)
            }
            EscrowInstruction::Exchange { amount } => {
                msg!("Instruction: Exchange");
                Self::process_exchange(accounts, amount, program_id)
            }
            EscrowInstruction::Cancel => {
                msg!("Instruction: Cancel");
                Self::process_cancel(accounts, program_id)
            }
        }
    }

    //* Create a platform state
    pub fn process_create_platform_state(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        //*update authority of platform
        let admin_update_auth =
            Pubkey::from_str("J8AjdAYf9jji6c8bnH56hwNHtdzovvJMjVmMBeDYY8uZ").unwrap();

        let account_info_iter = &mut accounts.iter();

        let admin = next_account_info(account_info_iter)?; // admin account key

        //* validation check if the user calling this instruction
        //** actually holds the authority for updating the platform account
        if admin_update_auth != *admin.key {
            msg!("Invalid admin.....");
            return Err(ProgramError::InvalidAccountData);
        }

        let platfrom_account = next_account_info(account_info_iter)?; // platform state account

        let system_program = next_account_info(account_info_iter)?; // system_program account

        let treasury_acc = next_account_info(account_info_iter)?; // treasury_acc account key

        //* Create a new account for platform state*/
        invoke(
            &create_account(
                admin.key,
                platfrom_account.key,
                Rent::default().minimum_balance(PlatformState::LEN),
                PlatformState::LEN as u64,
                program_id,
            ),
            &[
                admin.clone(),
                platfrom_account.clone(),
                system_program.clone(),
            ],
        )?;

        //* check if program owns platfrom_account account
        if platfrom_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        //* unpack the platfrom_account state, to store data into
        let mut account_update_info =
            PlatformState::unpack_unchecked(&platfrom_account.try_borrow_data()?)?;

        account_update_info.is_initialized = true;
        account_update_info.treasury_account = *treasury_acc.key;
        account_update_info.platform_fess = amount;

        //* pack data into the platform account
        PlatformState::pack(
            account_update_info,
            &mut platfrom_account.try_borrow_mut_data()?,
        )?;

        msg!("platfrom_state : {:?}", account_update_info);

        Ok(())
    }

    pub fn process_create_token_state(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        //*update authority of platform
        let admin_update_auth =
            Pubkey::from_str("J8AjdAYf9jji6c8bnH56hwNHtdzovvJMjVmMBeDYY8uZ").unwrap();

        let account_info_iter = &mut accounts.iter();

        let admin = next_account_info(account_info_iter)?; // admin account key

        //* validation check if the user calling this instruction
        //** actually holds the authority for updating the platform account
        if admin_update_auth != *admin.key {
            msg!("Invalid admin.....");
            return Err(ProgramError::InvalidAccountData);
        }

        let platfrom_account = next_account_info(account_info_iter)?; // platform state account

        //* check if program owns platfrom_account account
        if platfrom_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let token_state_account = next_account_info(account_info_iter)?; // token state account

        let token_mint = next_account_info(account_info_iter)?; // token mint

        let system_program = next_account_info(account_info_iter)?; // system_program account


        let (token_pda, nonce) = Pubkey::find_program_address(
            &[platfrom_account.key.as_ref(), token_mint.key.as_ref()],
            program_id,
        ); // Create pda for token state account

        if token_pda != *token_state_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        //* Create a new account for token state*/
        invoke_signed(
            &create_account(
                admin.key,
                token_state_account.key,
                Rent::default().minimum_balance(TokenState::LEN),
                TokenState::LEN as u64,
                program_id,
            ),
            &[
                admin.clone(),
                token_state_account.clone(),
                system_program.clone(),
            ],
            &[&[
                (platfrom_account.key).as_ref(),
                (token_mint.key).as_ref(),
                &[nonce],
            ]],
        )?;

        if token_state_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        //* unpack the platfrom_account state, to store data into
        let mut token_info = TokenState::unpack_unchecked(&token_state_account.try_borrow_data()?)?;

        token_info.is_initialized = true;
        token_info.owner_pubkey = *admin.key;
        token_info.total_no_of_tokens_listed = 0;
        token_info.token_mint = *token_mint.key;

        //* pack data into the platform account
        TokenState::pack(token_info, &mut token_state_account.try_borrow_mut_data()?)?;

        msg!("Token_state_info : {:?}", token_info);

        Ok(())
    }

    pub fn process_init_escrow(
        accounts: &[AccountInfo],
        args: (u64, u64),
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;
        // initializer is signer validation check
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let intializer_token_account = next_account_info(account_info_iter)?;

        let token_mint = next_account_info(account_info_iter)?;

        let pda_account = next_account_info(account_info_iter)?;

        let pda_token_account = next_account_info(account_info_iter)?;

        let token_program = next_account_info(account_info_iter)?;

        let system_program = next_account_info(account_info_iter)?;

        let token_state_account = next_account_info(account_info_iter)?;


        //* get a pda for escrow program
        let (pda, nonce) = Pubkey::find_program_address(
            &[initializer.key.as_ref(), token_mint.key.as_ref(),token_state_account.key.as_ref()],
            program_id,
        );

        if pda != *pda_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        //* Create a new account for Escrow
        if pda_account.owner != program_id {
            invoke_signed(
                &create_account(
                    initializer.key,
                    pda_account.key,
                    Rent::default().minimum_balance(ListerState::LEN),
                    ListerState::LEN as u64,
                    program_id,
                ),
                &[
                    initializer.clone(),
                    pda_account.clone(),
                    system_program.clone(),
                ],
                &[&[
                    (initializer.key).as_ref(),
                    (token_mint.key).as_ref(),
                    (token_state_account.key).as_ref(),
                    &[nonce],
                ]],
            )?;

            //*  Unpack escrow_account state
            let mut lister_info = ListerState::unpack_unchecked(&pda_account.try_borrow_data()?)?;

            lister_info.is_initialized = false;
        }

        let mut lister_info = ListerState::unpack_unchecked(&pda_account.try_borrow_data()?)?;
        let mut token_state_info = TokenState::unpack_unchecked(&token_state_account.try_borrow_data()?)?;

        if lister_info.is_initialized == true {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        //* Transfer token amount from initializer to pda
        let tranfer_instructions = spl_token::instruction::transfer(
            token_program.key,
            intializer_token_account.key,
            pda_token_account.key,
            initializer.key,
            &[],
            args.0 * 100,
        )?;
        invoke(
            &tranfer_instructions,
            &[
                intializer_token_account.clone(),
                pda_token_account.clone(),
                initializer.clone(),
                token_program.clone(),
            ],
        )?;

        //* set the state for escrow account
        lister_info.is_initialized = true;
        lister_info.seller_pubkey = *initializer.key;
        lister_info.token_mint = *token_mint.key;

        lister_info.token_account_pubkey = *pda_token_account.key;
        lister_info.token_amount = args.0;
        lister_info.expected_amount_per_token = args.1;

        token_state_info.total_no_of_tokens_listed=args.0;
        TokenState::pack(token_state_info, &mut token_state_account.try_borrow_mut_data()?)?;

        ListerState::pack(lister_info, &mut pda_account.try_borrow_mut_data()?)?;
        msg!("lister info {:?}", lister_info);
        Ok(())
    }

    pub fn process_exchange(
        accounts: &[AccountInfo],
        expected_token_amount_by_taker: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let taker = next_account_info(account_info_iter)?;

        //* check if the buyer is the singer for this instruction
        if !taker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let platform_state_account = next_account_info(account_info_iter)?;

        //* check if owner of platform account is the program
        if *platform_state_account.owner != *program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let pda_account = next_account_info(account_info_iter)?;

        let pdas_token_account = next_account_info(account_info_iter)?;

        let takers_token_account = next_account_info(account_info_iter)?;

        let initializers_main_account = next_account_info(account_info_iter)?;

        let token_mint = next_account_info(account_info_iter)?;

        let token_state_account = next_account_info(account_info_iter)?;


        let (pda, nonce) = Pubkey::find_program_address(
            &[
                initializers_main_account.key.as_ref(),
                token_mint.key.as_ref(),
                token_state_account.key.as_ref(),
            ],
            program_id,
        );

        if pda != *pda_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        //* check if owner of escrow account is the program
        if pda_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let mut lister_info = ListerState::unpack(&pda_account.try_borrow_data()?)?;
        let paltform_info = PlatformState::unpack(&platform_state_account.try_borrow_data()?)?;
        let mut token_state_info = TokenState::unpack(&token_state_account.try_borrow_data()?)?;

        let amount_per_token = lister_info.expected_amount_per_token;
        msg!("amount_per_token {}",amount_per_token);

        let total_sol_amount_of_tokens = expected_token_amount_by_taker * amount_per_token;

        msg!("total_sol_amount_of_tokens {}",total_sol_amount_of_tokens);


        let platform_fee = (total_sol_amount_of_tokens * paltform_info.platform_fess) / 100;

        msg!("platform_fee {}",platform_fee);


        let amount_expected_by_seller = total_sol_amount_of_tokens - platform_fee;

        msg!("amount_expected_by_seller {}",amount_expected_by_seller);


        if lister_info.seller_pubkey != *initializers_main_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let token_program = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        //* platform team and treasury accounts
        let platform_treasury = next_account_info(account_info_iter)?;


        //* validation checks for treasury and team accounts
        if paltform_info.treasury_account != *platform_treasury.key {
            return Err(ProgramError::InvalidAccountData);
        }

        //* transer SOL to initializers_main_account
        invoke(
            &transfer(
                taker.key,
                initializers_main_account.key,
                amount_expected_by_seller,
            ),
            &[
                taker.clone(),
                initializers_main_account.clone(),
                taker.clone(),
                system_program.clone(),
            ],
        )?;
        //* transer platform fees to treasury_account
        msg!("1");

        invoke(
            &transfer(taker.key, platform_treasury.key, platform_fee),
            &[
                taker.clone(),
                platform_treasury.clone(),
                taker.clone(),
                system_program.clone(),
            ],
        )?;
        msg!("2");
        //* transfer token to the buyer
        let tranfer_instructions = spl_token::instruction::transfer(
            token_program.key,
            pdas_token_account.key,
            takers_token_account.key,
            pda_account.key,
            &[],
            expected_token_amount_by_taker * 100,
        )?;
        invoke_signed(
            &tranfer_instructions,
            &[
                pdas_token_account.clone(),
                takers_token_account.clone(),
                pda_account.clone(),
            ],
            &[&[
                (initializers_main_account.key).as_ref(),
                (token_mint.key).as_ref(),
                (token_state_account.key).as_ref(),

                &[nonce],
            ]],
        )?;

        msg!("3");
        //* update the state of lister_info

        lister_info.token_amount = lister_info.token_amount - expected_token_amount_by_taker;

        token_state_info.total_no_of_tokens_listed = token_state_info.total_no_of_tokens_listed - expected_token_amount_by_taker;

        if lister_info.token_amount == 0 {
            lister_info.is_initialized = false;
        }

        ListerState::pack(lister_info, &mut pda_account.try_borrow_mut_data()?)?;
        TokenState::pack(token_state_info, &mut token_state_account.try_borrow_mut_data()?)?;

        msg!("4");

        Ok(())
    }

    pub fn process_cancel(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let user = next_account_info(account_info_iter)?;

        let token_mint = next_account_info(account_info_iter)?;

        let user_token_account = next_account_info(account_info_iter)?;

        let pdas_token_account = next_account_info(account_info_iter)?;

        let pda_account = next_account_info(account_info_iter)?;

        let token_state_account = next_account_info(account_info_iter)?;


        let (pda, nonce) =
            Pubkey::find_program_address(&[user.key.as_ref(), token_mint.key.as_ref(),token_state_account.key.as_ref()], program_id);

        if pda != *pda_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        //* check owner of escrow account is the program
        if pda_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        //* unpack the escrow state for some validation checks
        let mut lister_info = ListerState::unpack(&pda_account.try_borrow_data()?)?;

        //* check if the user cancelling the listing is actually
        //* the user who have listed it
        if lister_info.seller_pubkey != *user.key {
            return Err(ProgramError::InvalidAccountData);
        }
        if lister_info.token_account_pubkey != *pdas_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let token_program = next_account_info(account_info_iter)?;

        //* transfer the token amount back to the initializer

        let tranfer_instructions = spl_token::instruction::transfer(
            token_program.key,
            pdas_token_account.key,
            user_token_account.key,
            pda_account.key,
            &[],
            lister_info.token_amount * 100,
        )?;
        invoke_signed(
            &tranfer_instructions,
            &[
                pdas_token_account.clone(),
                user_token_account.clone(),
                pda_account.clone(),
            ],
            &[&[(user.key).as_ref(),(token_mint.key).as_ref(),token_state_account.key.as_ref(), &[nonce]]],
        )?;
        //* set the escorw state is_initialized to false

        lister_info.is_initialized = false;

        ListerState::pack(lister_info, &mut pda_account.try_borrow_mut_data()?)?;

        Ok(())
    }
}
