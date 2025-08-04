use anchor_lang::prelude::*;
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::state::{Mint, AccountState};
use spl_token_2022::instruction::{initialize_mint2, mint_to};
use anchor_lang::solana_program::{
    program::invoke,
    system_instruction,
    sysvar,
};
use spl_token_2022::extension::{
    transfer_fee::instruction::initialize_transfer_fee_config,
    memo_transfer::instruction::initialize_memo_transfer,
    default_account_state::instruction::initialize_default_account_state,
    immutable_owner::instruction::initialize_immutable_owner,
};

 use spl_associated_token_account::instruction::create_associated_token_account;
 use spl_associated_token_account::get_associated_token_address;



declare_id!("CswZd8SPTPqQHbEx8ZVdwxGwWBoWBvvewc6EFkEDtyLH");

#[program]
pub mod custom_token {
    use super::*;

    pub fn create_token_with_extensions(
        ctx: Context<CreateTokenWithExtensions>,
        args: CreateTokenArgs
    ) -> Result<()> {
        msg!("Creating token with extensions: {:?}", args);

        // Step 1: Calculate mint size
        let mint_size = get_mint_account_size(&args);

        // Step 2: Calculate rent
        let rent_lamports = Rent::get()?.minimum_balance(mint_size);

        // Step 3: Create mint account
        let create_ix = system_instruction::create_account(
            &ctx.accounts.payer.key(),
            &ctx.accounts.mint.key(),
            rent_lamports,
            mint_size as u64,
            &ctx.accounts.token_program.key(),
        );
        
       //We use invoke(...) because we’re calling low-level Solana programs, like Token-2022, System, etc. — not custom Anchor programs.
        invoke(
            &create_ix,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Step 4: Initialize base mint
        let init_mint_ix = initialize_mint2(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.mint_authority.key(),
            Some(&ctx.accounts.mint_authority.key()),
            args.decimals,
        )?;

        invoke(
            &init_mint_ix,
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;

        // Step 5: Initialize Transfer Fee (conditionally)
        if args.transfer_fee_basis_points > 0 {
            let init_fee_ix = initialize_transfer_fee_config(
                &ctx.accounts.token_program.key(),
                &ctx.accounts.mint.key(),
                Some(&ctx.accounts.mint_authority.key()),
                Some(&ctx.accounts.mint_authority.key()),
                args.transfer_fee_basis_points,
                args.max_fee,
            )?;

            invoke(
                &init_fee_ix,
                &[
                    ctx.accounts.mint.to_account_info(),
                    ctx.accounts.mint_authority.to_account_info(),
                ],
            )?;
        }

        // Step 6: Initialize Memo Transfer (conditionally)
        if args.require_memo {
            let memo_ix = initialize_memo_transfer(
                &ctx.accounts.token_program.key(),
                &ctx.accounts.mint.key(),
                Some(&ctx.accounts.mint_authority.key()),
                true,
            )?;

            invoke(
                &memo_ix,
                &[
                    ctx.accounts.mint.to_account_info(),
                    ctx.accounts.mint_authority.to_account_info(),
                ],
            )?;
        }

        // Step 7: Initialize DefaultAccountState (conditionally)
        if args.default_frozen {
            let freeze_ix = initialize_default_account_state(
                &ctx.accounts.token_program.key(),
                &ctx.accounts.mint.key(),
                Some(&ctx.accounts.mint_authority.key()),
                AccountState::Frozen,
            )?;

            invoke(
                &freeze_ix,
                &[
                    ctx.accounts.mint.to_account_info(),
                    ctx.accounts.mint_authority.to_account_info(),
                ],
            )?;
        }

        if args.immutable_owner {
           let ix = initialize_immutable_owner(
           &ctx.accounts.token_program.key(),
           &ctx.accounts.mint.key(),
         )?;

        invoke(
          &ix,
          &[ctx.accounts.mint.to_account_info()],
       )?;
} 


        if args.initial_supply > 0 {
    // Step 8: Check if the recipient_ata is valid
    let expected_ata = get_associated_token_address(
        &ctx.accounts.recipient.key(),
        &ctx.accounts.mint.key(),
    );

    require_keys_eq!(
        expected_ata,
        ctx.accounts.recipient_ata.key(),
        CustomTokenError::InvalidAta
    );

    // Step 9: Create ATA if it's empty (i.e., doesn't exist yet)
    if ctx.accounts.recipient_ata.data_is_empty() {
        let create_ata_ix = create_associated_token_account(
            &ctx.accounts.payer.key(),
            &ctx.accounts.recipient.key(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.token_program.key(),
        );

           invoke(
              &create_ata_ix,
               &[
                   ctx.accounts.payer.to_account_info(),
                   ctx.accounts.recipient_ata.to_account_info(),
                   ctx.accounts.recipient.to_account_info(),
                   ctx.accounts.mint.to_account_info(),
                   ctx.accounts.system_program.to_account_info(),
                   ctx.accounts.token_program.to_account_info(),
                   ctx.accounts.rent.to_account_info(),
               ],
           )?;
       }

       // Step 10: Mint tokens
       let mint_to_ix = mint_to(
           &ctx.accounts.token_program.key(),
           &ctx.accounts.mint.key(),
           &ctx.accounts.recipient_ata.key(),
           &ctx.accounts.mint_authority.key(),
           &[],
           args.initial_supply,
       )?;

       invoke(
           &mint_to_ix,
           &[
               ctx.accounts.mint.to_account_info(),
               ctx.accounts.recipient_ata.to_account_info(),
               ctx.accounts.mint_authority.to_account_info(),
           ],
       )?;
    }

        Ok(())
    }

    pub fn get_mint_account_size(args: &CreateTokenArgs) -> usize {
        let mut extensions = vec![
            ExtensionType::TransferFeeConfig,
            ExtensionType::TransferFeeAmount,
        ];

        if args.require_memo {
            extensions.push(ExtensionType::MemoTransfer);
        }

        if args.default_frozen {
            extensions.push(ExtensionType::DefaultAccountState);
        }

        if args.immutable_owner {
            extensions.push(ExtensionType::ImmutableOwner);
        }

        ExtensionType::try_calculate_account_len::<Mint>(&extensions).unwrap()
    }
}


#[derive(Accounts)]
pub struct CreateTokenWithExtensions<'info> {
    #[account]
    pub payer: Signer<'info>,

    ///CHECK: we will create this manually using CPI
    #[account(mut)]
    pub mint : UncheckedAccount<'info>,

    ///CHECK:will be set as the mint authority
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: the user’s wallet who will receive the initial supply
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: ATA will be created if needed
    #[account(mut)]
    pub recipient_ata: UncheckedAccount<'info>,


    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>, 
}


//not a data account, just a struct to hold the arguments
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreateTokenArgs {
    pub decimals: u8,
    pub transfer_fee_basis_points: u16,
    pub max_fee: u64,
    pub require_memo: bool,
    pub default_frozen: bool,
    pub immutable_owner: bool,
    pub initial_supply: u64,
}


#[error_code]
pub enum CustomTokenError {
    #[msg("The provided associated token account is incorrect.")]
    InvalidAta,
}
