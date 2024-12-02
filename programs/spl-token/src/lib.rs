use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{mpl_token_metadata::types::DataV2, Metadata as Metaplex},
    token::{
        spl_token::instruction::AuthorityType, thaw_account, Mint, ThawAccount, Token, TokenAccount,
    },
};

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("DvM3Npn4ZnsuSptoRCu8U1b8hLAFww2xWmpjvKzndKjH");

#[program]
mod spl_token {
    use super::*;

    pub fn create_token(ctx: Context<CreateToken>, params: MetadataParams) -> Result<()> {
        let seeds = &["mint".as_bytes(), &[ctx.bumps.mint]];
        let signer = [&seeds[..]];

        let token_data: DataV2 = DataV2 {
            name: params.name,
            symbol: params.symbol,
            uri: params.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        // create metadata account
        let metadata_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            anchor_spl::metadata::CreateMetadataAccountsV3 {
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.mint.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                mint_authority: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer,
        );

        anchor_spl::metadata::create_metadata_accounts_v3(
            metadata_ctx,
            token_data,
            params.is_mutable,
            true,
            None,
        )?;

        Ok(())
    }

    pub fn mint_token(ctx: Context<MintToken>, quantity: u64) -> Result<()> {
        let seeds = &["mint".as_bytes(), &[ctx.bumps.mint]];
        let signer = [&seeds[..]];

        anchor_spl::token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    authority: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
                &signer,
            ),
            quantity,
        )?;

        Ok(())
    }

    pub fn transer_token(ctx: Context<TransferToken>, amount: u64) -> Result<()> {
        msg!(
            "Started {:} tokens transfer from account {:} to {:}",
            amount,
            ctx.accounts.from_account.key(),
            ctx.accounts.to_account.key()
        );

        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    authority: ctx.accounts.signer.to_account_info(),
                    from: ctx.accounts.from_account.to_account_info(),
                    to: ctx.accounts.to_account.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn set_authority_token(ctx: Context<SetAuthorityToken>, authority_value: u8) -> Result<()> {
        let signer = ctx.accounts.signer.key();

        let account_or_mint;
        let authority_type;
        match authority_value {
            0 => {
                authority_type = AuthorityType::MintTokens;
                account_or_mint = ctx.accounts.mint.to_account_info();
            }
            1 => {
                authority_type = AuthorityType::FreezeAccount;
                account_or_mint = ctx.accounts.mint.to_account_info();
            }
            2 => {
                authority_type = AuthorityType::AccountOwner;
                account_or_mint = ctx.accounts.token_account.to_account_info();
            }
            _ => {
                authority_type = AuthorityType::CloseAccount;
                account_or_mint = ctx.accounts.token_account.to_account_info();
            }
        }
        anchor_spl::token::set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::SetAuthority {
                    account_or_mint: account_or_mint,
                    current_authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            authority_type.clone(),
            Some(signer),
        )?;

        Ok(())
    }

    pub fn burn_token(ctx: Context<BurnToken>, amount: u64) -> Result<()> {
        anchor_spl::token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Burn {
                    authority: ctx.accounts.signer.to_account_info(),
                    from: ctx.accounts.token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }

    pub fn freeze_token(ctx: Context<FreezeToken>) -> Result<()> {
        anchor_spl::token::freeze_account(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::FreezeAccount {
                account: ctx.accounts.token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ))?;

        Ok(())
    }

    pub fn unfreeze_token(ctx: Context<FreezeToken>) -> Result<()> {
        thaw_account(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            ThawAccount {
                account: ctx.accounts.token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ))?;

        Ok(())
    }

    pub fn close_token(ctx: Context<CloseToken>) -> Result<()> {
        anchor_spl::token::close_account(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: ctx.accounts.token_account.to_account_info(),
                destination: ctx.accounts.signer.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ))?;

        Ok(())
    }

    pub fn set_token_metadata(ctx: Context<CreateMetadata>, data: MetadataParams) -> Result<()> {
        let (metadata_address, _b1) = Pubkey::find_program_address(
            &[
                b"metadata",
                &ctx.accounts.metadata_program.key.to_bytes(),
                &ctx.accounts.mint.key().to_bytes(),
            ],
            ctx.accounts.metadata_program.key,
        );

        let metadata_account = &ctx.accounts.metadata_account;
        let master_account = &ctx.accounts.master_account;

        if metadata_address != *metadata_account.key {
            return err!(ProgramErrors::PdaNotMatched);
        }

        let (master_address, _b2) = Pubkey::find_program_address(
            &[
                b"metadata",
                &ctx.accounts.metadata_program.key.to_bytes(),
                &ctx.accounts.mint.key().to_bytes(),
                b"edition",
            ],
            ctx.accounts.metadata_program.key,
        );

        if master_address != *master_account.key {
            return err!(ProgramErrors::PdaNotMatched);
        }

        anchor_spl::metadata::create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.metadata_program.to_account_info(),
                anchor_spl::metadata::CreateMetadataAccountsV3 {
                    metadata: metadata_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    mint_authority: ctx.accounts.signer.to_account_info(),
                    update_authority: ctx.accounts.signer.to_account_info(),
                    payer: ctx.accounts.signer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            DataV2 {
                name: data.name,
                symbol: data.symbol,
                uri: data.uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            data.is_mutable,
            true,
            None,
        )?;

        Ok(())
    }

}

#[derive(Accounts)]
#[instruction( params: MetadataParams )]
pub struct CreateToken<'info> {
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    #[account(
        init,
        seeds = [b"mint"],
        bump,
        payer = payer,
        mint::decimals = params.decimals,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub token_metadata_program: Program<'info, Metaplex>,
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub destination: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct TransferToken<'info> {
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub from_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associate_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct SetAuthorityToken<'info> {
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub new_signer: Signer<'info>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnToken<'info> {
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct FreezeToken<'info> {
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CloseToken<'info> {
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateMetadata<'info> {
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub metadata_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub master_account: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub edition_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associate_token_program: Program<'info, AssociatedToken>,
    /// CHECK:
    pub metadata_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct MetadataParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
    pub is_mutable: bool,
}

#[error_code]
pub enum ProgramErrors {
    #[msg("PDA account not matched")]
    PdaNotMatched,
}
