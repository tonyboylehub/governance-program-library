use crate::program_test::core_voter_test::ConfigureCollectionArgs;
use gpl_core_voter::error::NftVoterError;
use program_test::core_voter_test::{CastAssetVoteArgs, CoreVoterTest};
use program_test::tools::{assert_gov_err, assert_nft_voter_err};
use solana_program_test::*;
use solana_sdk::transport::TransportError;
use spl_governance::error::GovernanceError;

mod program_test;

async fn advance_clock_multiple_times(core_voter_test: &mut CoreVoterTest, times: u32) {
    for _ in 0..times {
        core_voter_test.bench.advance_clock().await;
    }
}

#[tokio::test]
async fn test_relinquish_nft_vote() -> Result<(), TransportError> {
    // Arrange
    let mut core_voter_test = CoreVoterTest::start_new().await;

    let realm_cookie = core_voter_test.governance.with_realm().await?;

    let registrar_cookie = core_voter_test.with_registrar(&realm_cookie).await?;

    let collection_cookie = core_voter_test.core.create_collection(None).await?;

    let mut max_voter_weight_record_cookie = core_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let voter_cookie = core_voter_test.bench.with_wallet().await;

    let asset_cookie1 = core_voter_test
        .core
        .create_asset(&collection_cookie, &voter_cookie)
        .await?;

    core_voter_test
        .with_collection(
            &registrar_cookie,
            &collection_cookie,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs { weight: 1 }), // Set Size == 1 to complete voting with just one vote
        )
        .await?;

    let voter_token_owner_record_cookie = core_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = core_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = core_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    // Update max voter weight record before casting vote
    core_voter_test
        .update_max_voter_weight_record(&registrar_cookie, &mut max_voter_weight_record_cookie)
        .await?;

    let asset_vote_record_cookies = core_voter_test
        .cast_asset_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&asset_cookie1],
            None,
        )
        .await?;

    // Advance clock more times to ensure expiry
    advance_clock_multiple_times(&mut core_voter_test, 12).await;

    // Act
    core_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &asset_vote_record_cookies,
        )
        .await?;

    // Assert
    let voter_weight_record = core_voter_test
        .get_voter_weight_record(&voter_weight_record_cookie.address)
        .await;

    assert_eq!(voter_weight_record.voter_weight_expiry, Some(0));
    assert_eq!(voter_weight_record.voter_weight, 0);

    // Check NftVoteRecord was disposed
    let asset_vote_record = core_voter_test
        .bench
        .get_account(&asset_vote_record_cookies[0].address)
        .await;

    assert_eq!(None, asset_vote_record);

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_for_proposal_in_voting_state() -> Result<(), TransportError> {
    // Arrange
    let mut core_voter_test = CoreVoterTest::start_new().await;

    let realm_cookie = core_voter_test.governance.with_realm().await?;

    let registrar_cookie = core_voter_test.with_registrar(&realm_cookie).await?;

    let collection_cookie = core_voter_test.core.create_collection(None).await?;

    let mut max_voter_weight_record_cookie = core_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let voter_cookie = core_voter_test.bench.with_wallet().await;

    let asset_cookie1 = core_voter_test
        .core
        .create_asset(&collection_cookie, &voter_cookie)
        .await?;

    core_voter_test
        .with_collection(
            &registrar_cookie,
            &collection_cookie,
            &max_voter_weight_record_cookie,
            None,
        )
        .await?;

    let voter_token_owner_record_cookie = core_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = core_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = core_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    // Update max voter weight record before casting vote
    core_voter_test
        .update_max_voter_weight_record(&registrar_cookie, &mut max_voter_weight_record_cookie)
        .await?;

    let asset_vote_record_cookies = core_voter_test
        .cast_asset_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&asset_cookie1],
            None,
        )
        .await?;

    advance_clock_multiple_times(&mut core_voter_test, 6).await;

    // Relinquish Vote from spl-gov
    core_voter_test
        .governance
        .relinquish_vote(
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
        )
        .await?;

    advance_clock_multiple_times(&mut core_voter_test, 6).await;

    // Act
    core_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &asset_vote_record_cookies,
        )
        .await?;

    // Assert
    let voter_weight_record = core_voter_test
        .get_voter_weight_record(&voter_weight_record_cookie.address)
        .await;

    assert_eq!(voter_weight_record.voter_weight_expiry, Some(0));
    assert_eq!(voter_weight_record.voter_weight, 0);

    // Check NftVoteRecord was disposed
    let asset_vote_record = core_voter_test
        .bench
        .get_account(&asset_vote_record_cookies[0].address)
        .await;

    assert_eq!(None, asset_vote_record);

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_for_proposal_in_voting_state_and_vote_record_exists_error() -> Result<(), TransportError> {
    // Arrange
    let mut core_voter_test = CoreVoterTest::start_new().await;

    let realm_cookie = core_voter_test.governance.with_realm().await?;

    let registrar_cookie = core_voter_test.with_registrar(&realm_cookie).await?;

    let collection_cookie = core_voter_test.core.create_collection(Some(5)).await?;

    let mut max_voter_weight_record_cookie = core_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let voter_cookie = core_voter_test.bench.with_wallet().await;

    // Create and store 6th asset. This will be used for the vote.
    let asset_cookie1 = core_voter_test
        .core
        .create_asset(&collection_cookie, &voter_cookie)
        .await?;

    core_voter_test
        .with_collection(
            &registrar_cookie,
            &collection_cookie,
            &max_voter_weight_record_cookie,
            None,
        )
        .await?;

    let voter_token_owner_record_cookie = core_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = core_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = core_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    // Update max voter weight record before casting vote
    core_voter_test
        .update_max_voter_weight_record(&registrar_cookie, &mut max_voter_weight_record_cookie)
        .await?;

    let asset_vote_record_cookies = core_voter_test
        .cast_asset_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&asset_cookie1],
            None,
        )
        .await?;

    advance_clock_multiple_times(&mut core_voter_test, 6).await;

    // Act
    let err = core_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &asset_vote_record_cookies,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_nft_voter_err(err, NftVoterError::VoteRecordMustBeWithdrawn);

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_with_invalid_voter_error() -> Result<(), TransportError> {
    // Arrange
    let mut core_voter_test = CoreVoterTest::start_new().await;

    let realm_cookie = core_voter_test.governance.with_realm().await?;

    let registrar_cookie = core_voter_test.with_registrar(&realm_cookie).await?;

    let collection_cookie = core_voter_test.core.create_collection(None).await?;

    let mut max_voter_weight_record_cookie = core_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let voter_cookie = core_voter_test.bench.with_wallet().await;

    let asset_cookie1 = core_voter_test
        .core
        .create_asset(&collection_cookie, &voter_cookie)
        .await?;

    core_voter_test
        .with_collection(
            &registrar_cookie,
            &collection_cookie,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs { weight: 1 }), // Set Size == 1 to complete voting with just one vote
        )
        .await?;

    let voter_token_owner_record_cookie = core_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = core_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = core_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    // Update max voter weight record before casting vote
    core_voter_test
        .update_max_voter_weight_record(&registrar_cookie, &mut max_voter_weight_record_cookie)
        .await?;

    let asset_vote_record_cookies = core_voter_test
        .cast_asset_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&asset_cookie1],
            None,
        )
        .await?;

    // Try to use a different voter
    let voter_cookie2 = core_voter_test.bench.with_wallet().await;

    // Act

    let err = core_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie2,
            &voter_token_owner_record_cookie,
            &asset_vote_record_cookies,
        )
        .await
        .err()
        .unwrap();

    // Assert

    assert_gov_err(err, GovernanceError::GoverningTokenOwnerOrDelegateMustSign);

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_with_unexpired_vote_weight_record() -> Result<(), TransportError>
{
    // Arrange
    let mut core_voter_test = CoreVoterTest::start_new().await;

    let realm_cookie = core_voter_test.governance.with_realm().await?;

    let registrar_cookie = core_voter_test.with_registrar(&realm_cookie).await?;

    let collection_cookie = core_voter_test.core.create_collection(None).await?;

    let mut max_voter_weight_record_cookie = core_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let voter_cookie = core_voter_test.bench.with_wallet().await;

    let asset_cookie1 = core_voter_test
        .core
        .create_asset(&collection_cookie, &voter_cookie)
        .await?;

    core_voter_test
        .with_collection(
            &registrar_cookie,
            &collection_cookie,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs { weight: 10 }),
        )
        .await?;

    let voter_token_owner_record_cookie = core_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = core_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = core_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    // Update max voter weight record before casting vote
    core_voter_test
        .update_max_voter_weight_record(&registrar_cookie, &mut max_voter_weight_record_cookie)
        .await?;

    let args = CastAssetVoteArgs {
        cast_spl_gov_vote: false,
    };

    // Cast vote with NFT
    let asset_vote_record_cookies = core_voter_test
        .cast_asset_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&asset_cookie1],
            Some(args),
        )
        .await?;

    // Act
    let err = core_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &asset_vote_record_cookies,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_nft_voter_err(err, NftVoterError::VoterWeightRecordMustBeExpired);

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_with_invalid_voter_weight_token_owner_error() -> Result<(), TransportError> {
    // Arrange
    let mut core_voter_test = CoreVoterTest::start_new().await;

    let realm_cookie = core_voter_test.governance.with_realm().await?;

    let registrar_cookie = core_voter_test.with_registrar(&realm_cookie).await?;

    let collection_cookie = core_voter_test.core.create_collection(None).await?;

    let mut max_voter_weight_record_cookie = core_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let voter_cookie = core_voter_test.bench.with_wallet().await;

    let asset_cookie1 = core_voter_test
        .core
        .create_asset(&collection_cookie, &voter_cookie)
        .await?;

    core_voter_test
        .with_collection(
            &registrar_cookie,
            &collection_cookie,
            &max_voter_weight_record_cookie,
            None,
        )
        .await?;

    let voter_token_owner_record_cookie = core_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = core_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = core_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    // Update max voter weight record before casting vote
    core_voter_test
        .update_max_voter_weight_record(&registrar_cookie, &mut max_voter_weight_record_cookie)
        .await?;

    let asset_vote_record_cookies = core_voter_test
        .cast_asset_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&asset_cookie1],
            None,
        )
        .await?;

    // Try to update VoterWeightRecord for different governing_token_owner
    let voter_cookie2 = core_voter_test.bench.with_wallet().await;
    let voter_weight_record_cookie2 = core_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie2)
        .await?;

    // Act

    let err = core_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie2,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &asset_vote_record_cookies,
        )
        .await
        .err()
        .unwrap();

    // Assert

    assert_nft_voter_err(err, NftVoterError::InvalidTokenOwnerForVoterWeightRecord);

    Ok(())
}

#[tokio::test]
async fn test_relinquish_nft_vote_using_delegate() -> Result<(), TransportError> {
    // Arrange
    let mut core_voter_test = CoreVoterTest::start_new().await;

    let realm_cookie = core_voter_test.governance.with_realm().await?;

    let registrar_cookie = core_voter_test.with_registrar(&realm_cookie).await?;

    let collection_cookie = core_voter_test.core.create_collection(None).await?;

    let mut max_voter_weight_record_cookie = core_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let voter_cookie = core_voter_test.bench.with_wallet().await;

    let asset_cookie1 = core_voter_test
        .core
        .create_asset(&collection_cookie, &voter_cookie)
        .await?;

    core_voter_test
        .with_collection(
            &registrar_cookie,
            &collection_cookie,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs { weight: 1 }), // Set Size == 1 to complete voting with just one vote
        )
        .await?;

    let voter_token_owner_record_cookie = core_voter_test
        .governance
        .with_token_owner_record(&realm_cookie, &voter_cookie)
        .await?;

    let voter_weight_record_cookie = core_voter_test
        .with_voter_weight_record(&registrar_cookie, &voter_cookie)
        .await?;

    let proposal_cookie = core_voter_test
        .governance
        .with_proposal(&realm_cookie)
        .await?;

    // Update max voter weight record before casting vote
    core_voter_test
        .update_max_voter_weight_record(&registrar_cookie, &mut max_voter_weight_record_cookie)
        .await?;

    let asset_vote_record_cookies = core_voter_test
        .cast_asset_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &max_voter_weight_record_cookie,
            &proposal_cookie,
            &voter_cookie,
            &voter_token_owner_record_cookie,
            &[&asset_cookie1],
            None,
        )
        .await?;

    advance_clock_multiple_times(&mut core_voter_test, 6).await;

    // Setup delegate
    let delegate_cookie = core_voter_test.bench.with_wallet().await;
    core_voter_test
        .governance
        .set_governance_delegate(
            &realm_cookie,
            &voter_token_owner_record_cookie,
            &voter_cookie,
            &Some(delegate_cookie.address),
        )
        .await;

    // Print current slot and expiry before advancing
    // {
    //     let current_slot = core_voter_test.bench.get_clock().await.slot;
    //     let voter_weight_record = core_voter_test.get_voter_weight_record(&voter_weight_record_cookie.address).await;
    //     println!("Before advancing: current_slot = {}, expiry = {:?}", current_slot, voter_weight_record.voter_weight_expiry);
    // }

    // Advance clock only once so the record is NOT expired
    // advance_clock_multiple_times(&mut core_voter_test, 1).await;

    // Print current slot and expiry after advancing
    // {
    //     let current_slot = core_voter_test.bench.get_clock().await.slot;
    //     let voter_weight_record = core_voter_test.get_voter_weight_record(&voter_weight_record_cookie.address).await;
    //     println!("After advancing: current_slot = {}, expiry = {:?}", current_slot, voter_weight_record.voter_weight_expiry);
    // }

    // Act

    core_voter_test
        .relinquish_nft_vote(
            &registrar_cookie,
            &voter_weight_record_cookie,
            &proposal_cookie,
            &delegate_cookie,
            &voter_token_owner_record_cookie,
            &asset_vote_record_cookies,
        )
        .await?;

    // Assert

    let voter_weight_record = core_voter_test
        .get_voter_weight_record(&voter_weight_record_cookie.address)
        .await;

    assert_eq!(voter_weight_record.voter_weight_expiry, Some(0));
    assert_eq!(voter_weight_record.voter_weight, 0);

    // Check NftVoteRecord was disposed
    let asset_vote_record = core_voter_test
        .bench
        .get_account(&asset_vote_record_cookies[0].address)
        .await;

    assert_eq!(None, asset_vote_record);

    Ok(())
}
