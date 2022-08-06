pub mod error;
pub mod state;

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{CloseAccount, Mint, Token, Transfer};

use crate::error::ErrorCode;
use crate::state::Stage;
declare_id!("Wk1uGMfZR6YhTjLAaUD1e944VcrgKvZXsFVPonjy1yD");

#[program]
pub mod transfer {
    use super::*;

    pub fn deposit(
        ctx: Context<DepositInstruction>,
        amount: u64,
        stage_bump: u8,
        wallet_bump: u8,
    ) -> Result<()> {
        let state = &mut ctx.accounts.state_account;
        state.user = ctx.accounts.user.key().clone();
        state.mint = ctx.accounts.mint.key().clone();
        state.escrow_wallet = ctx.accounts.escrow_wallet_associate_account.key().clone();
        state.amount = amount;

        let bump_vector = stage_bump.to_le_bytes();
        let mint_token = ctx.accounts.mint.key().clone();

        let inner = vec![
            b"state".as_ref(),
            ctx.accounts.user.key.as_ref(),
            mint_token.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        // init account
        let transfer_instruction = Transfer {
            from: ctx.accounts.user_associated_account.to_account_info(),
            to: ctx
                .accounts
                .escrow_wallet_associate_account
                .to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer.as_slice(),
        );

        anchor_spl::token::transfer(cpi_ctx, state.amount)?;

        state.stage = Stage::Deposit.to_code();
        Ok(())
    }

    // pub fn with_draw(
    //     ctx: Context<WithDrawInstruction>,
    //     state_bump: u8,
    //     wallet_bump: u8,
    // ) -> Result<()> {
    //     let current_stage = Stage::from(ctx.accounts.state_account.stage)?;
    //     let is_valid_stage = current_stage == Stage::Deposit || current_stage == Stage::WithDraw;
    //     if !is_valid_stage {
    //         return Err(ErrorCode::InvalidStage.into());
    //     }

    //     let bump_vector = state_bump.to_le_bytes();
    //     let mint_token = ctx.accounts.mint.key().clone();
    //     let inner = vec![
    //         b"state".as_ref(),
    //         ctx.accounts.user.key.as_ref(),
    //         mint_token.as_ref(),
    //         bump_vector.as_ref(),
    //     ];
    //     let outer = vec![inner.as_slice()];

    //     let transfer_instruction = Transfer {
    //         from: ctx
    //             .accounts
    //             .escrow_wallet_associate_account
    //             .to_account_info(),
    //         to: ctx.accounts.user_associated_account.to_account_info(),
    //         authority: ctx.accounts.state_account.to_account_info(),
    //     };
    //     let cpi_ctx = CpiContext::new_with_signer(
    //         ctx.accounts.token_program.to_account_info(),
    //         transfer_instruction,
    //         outer.as_slice(),
    //     );
    //     anchor_spl::token::transfer(cpi_ctx, ctx.accounts.escrow_wallet_associate_account.amount)?;

    //     // close Account when dont use any where again
    //     let should_close = {
    //         ctx.accounts.escrow_wallet_associate_account.reload()?;
    //         ctx.accounts.escrow_wallet_associate_account.amount == 0
    //     };

    //     if should_close {
    //         let ca = CloseAccount {
    //             account: ctx
    //                 .accounts
    //                 .escrow_wallet_associate_account
    //                 .to_account_info(),
    //             destination: ctx.accounts.user.to_account_info(),
    //             authority: ctx.accounts.state_account.to_account_info(),
    //         };
    //         let cpi_ctx = CpiContext::new_with_signer(
    //             ctx.accounts.token_program.to_account_info(),
    //             ca,
    //             outer.as_slice(),
    //         );
    //         anchor_spl::token::close_account(cpi_ctx)?;
    //     }

    //     Ok(())
    // }
}

#[account]
pub struct State {
    user: Pubkey,
    mint: Pubkey,
    // associated account
    escrow_wallet: Pubkey,
    amount: u64,
    stage: u8,
}

#[derive(Accounts)]
#[instruction(bump: u8, wallet_bump: u8)]
pub struct DepositInstruction<'info> {
    #[account(
        init,
        payer = user,
        seeds=[b"state".as_ref(), user.key().as_ref(), mint.key().as_ref()],
        bump,
        space = 132
    )]
    state_account: Account<'info, State>,
    #[account(
        mut,
        seeds=[b"wallet".as_ref(), user.key().as_ref(), mint.key().as_ref()],
        bump = wallet_bump,
        token::mint=mint,
        token::authority=state_account,
    )]
    escrow_wallet_associate_account: Account<'info, TokenAccount>,

    #[account(mut)]
    user: Signer<'info>,
    mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint=user_associated_account.owner == user.key(),
        constraint=user_associated_account.mint == mint.key()
    )]
    user_associated_account: Account<'info, TokenAccount>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

// #[derive(Accounts)]
// #[instruction(state_bump: u8, wallet_bump: u8)]
// pub struct WithDrawInstruction<'info> {
//     #[account(
//         mut,
//         seeds=[b"state".as_ref(), user.key().as_ref(), mint.key().as_ref()],
//         bump = state_bump,
//         has_one = user,
//         has_one = mint,
//     )]
//     state_account: Account<'info, State>,
//     #[account(
//         mut,
//         seeds=[b"wallet".as_ref(), user.key().as_ref(), mint.key().as_ref()],
//         bump = wallet_bump,
//     )]
//     escrow_wallet_associate_account: Account<'info, TokenAccount>,
//     #[account(mut)]
//     user: Signer<'info>,
//     mint: Account<'info, Mint>,
//     // refund wallet
//     #[account(
//         mut,
//         constraint=user_associated_account.owner == user.key(),
//         constraint=user_associated_account.mint == mint.key()
//     )]
//     user_associated_account: Account<'info, TokenAccount>,
//     // system
//     system_program: Program<'info, System>,
//     token_program: Program<'info, Token>,
// }
