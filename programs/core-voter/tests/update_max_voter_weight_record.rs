use crate::program_test::core_voter_test::ConfigureCollectionArgs;
use program_test::core_voter_test::CoreVoterTest;
use solana_program_test::*;
use solana_sdk::transport::TransportError;

mod program_test;

#[tokio::test]
async fn test_update_collection_config_invalidates_max_voter_weight_record_expirey(
) -> Result<(), TransportError> {
    // Arrange
    let mut core_voter_test = CoreVoterTest::start_new().await;

    let realm_cookie = core_voter_test.governance.with_realm().await?;

    let registrar_cookie = core_voter_test.with_registrar(&realm_cookie).await?;

    let _voter_cookie = core_voter_test.bench.with_wallet().await;

    let mut max_voter_weight_record_cookie = core_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let collection_1_size = 7;
    let collection_1_weight = 5;

    let collection_cookie_1 = core_voter_test
        .core
        .create_collection(Some(collection_1_size))
        .await?;

    // Register collection_1 to registrar
    core_voter_test
        .with_collection(
            &registrar_cookie,
            &collection_cookie_1,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs {
                weight: collection_1_weight,
            }),
        )
        .await?;

    // Verify that max_voter_weight_expiry is Some(0) and max_voter_weight is 0 after configuring collection
    let max_voter_weight_record = core_voter_test
        .get_max_voter_weight_record(&max_voter_weight_record_cookie.address)
        .await;
    assert_eq!(max_voter_weight_record.max_voter_weight_expiry, Some(0));
    assert_eq!(max_voter_weight_record.max_voter_weight, 0);

    // Generate an updated new max voter weight record
    core_voter_test
        .update_max_voter_weight_record(&registrar_cookie, &mut max_voter_weight_record_cookie)
        .await?;

    // Verify that max_voter_weight_expiry is Some(current_slot) and max_voter_weight is correct after updating
    let max_voter_weight_record = core_voter_test
        .get_max_voter_weight_record(&max_voter_weight_record_cookie.address)
        .await;
    let current_slot = core_voter_test.bench.get_clock().await.slot;
    assert_eq!(max_voter_weight_record.max_voter_weight_expiry, Some(current_slot));
    assert_eq!(max_voter_weight_record.max_voter_weight, (collection_1_weight * collection_1_size) as u64);

    let collection_2_size = 10;
    let collection_2_weight = 2;

    // Generate a new collection and update the registrar with the additional collection
    // while invalidating max voter weight.
    let collection_cookie_2 = core_voter_test
        .core
        .create_collection(Some(collection_2_size))
        .await?;

    // Register collection_2 to registrar
    core_voter_test
        .with_collection(
            &registrar_cookie,
            &collection_cookie_2,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs {
                weight: collection_2_weight,
            }),
        )
        .await?;

    // Verify that max_voter_weight_expiry is Some(0) and max_voter_weight is 0 after configuring collection_2
    let max_voter_weight_record = core_voter_test
        .get_max_voter_weight_record(&max_voter_weight_record_cookie.address)
        .await;
    assert_eq!(max_voter_weight_record.max_voter_weight_expiry, Some(0));
    assert_eq!(max_voter_weight_record.max_voter_weight, 0);

    // Fetch registrar account and assert that collection was added to the registrars collection_configs
    let registrar = core_voter_test
        .get_registrar_account(&registrar_cookie.address)
        .await;

    let max_voter_weight_record = core_voter_test
        .get_max_voter_weight_record(&max_voter_weight_record_cookie.address)
        .await;

    // Assert
    let max_voter_weight_total =
        (collection_1_weight * collection_1_size) + (collection_2_weight * collection_2_size);

    assert!(registrar.collection_configs.len() == 2);
    assert_eq!(max_voter_weight_record.max_voter_weight_expiry, Some(0));
    assert_eq!(max_voter_weight_record.max_voter_weight, 0);

    Ok(())
}

#[tokio::test]
async fn test_update_max_voter_weight_record_provides_valid_expirey() -> Result<(), TransportError>
{
    // Arrange
    let mut core_voter_test = CoreVoterTest::start_new().await;

    let realm_cookie = core_voter_test.governance.with_realm().await?;

    let registrar_cookie = core_voter_test.with_registrar(&realm_cookie).await?;

    let mut max_voter_weight_record_cookie = core_voter_test
        .with_max_voter_weight_record(&registrar_cookie)
        .await?;

    let _voter_cookie = core_voter_test.bench.with_wallet().await;

    // Set collection sizes and weights for collection_1
    let collection_1_size = 11;
    let collection_1_weight = 4;

    let collection_cookie_1 = core_voter_test
        .core
        .create_collection(Some(collection_1_size))
        .await?;

    // Register collection_1 to registrar
    let _collection_config_cookie = core_voter_test
        .with_collection(
            &registrar_cookie,
            &collection_cookie_1,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs {
                weight: collection_1_weight,
            }),
        )
        .await?;

    // Verify that max_voter_weight_expiry is Some(0) after configuring collection
    let max_voter_weight_record = core_voter_test
        .get_max_voter_weight_record(&max_voter_weight_record_cookie.address)
        .await;
    assert_eq!(max_voter_weight_record.max_voter_weight_expiry, Some(0));

    // Generate an updated new max voter weight record
    core_voter_test
        .update_max_voter_weight_record(&registrar_cookie, &mut max_voter_weight_record_cookie)
        .await?;

    // Verify that max_voter_weight_expiry is Some(current_slot) after updating
    let max_voter_weight_record = core_voter_test
        .get_max_voter_weight_record(&max_voter_weight_record_cookie.address)
        .await;
    let current_slot = core_voter_test.bench.get_clock().await.slot;
    assert_eq!(max_voter_weight_record.max_voter_weight_expiry, Some(current_slot));

    // Advance clock so that second `update_max_voter_weight_record`` can be made without a duplicate
    // transaction submission which causes transaction to "pass" but not actually update the account.
    core_voter_test.bench.advance_clock().await;
    let _clock = core_voter_test.bench.get_clock().await;

    // Generate a new collection and update the registrar with the additional collection
    // which also invalidates max_voter_weight_expirey.
    let collection_2_size = 9;
    let collection_2_weight = 3;
    let collection_cookie_2 = core_voter_test
        .core
        .create_collection(Some(collection_2_size))
        .await?;

    // Register collection_2 to registrar
    core_voter_test
        .with_collection(
            &registrar_cookie,
            &collection_cookie_2,
            &max_voter_weight_record_cookie,
            Some(ConfigureCollectionArgs {
                weight: collection_2_weight,
            }),
        )
        .await?;

    // Verify that max_voter_weight_expiry is Some(0) after configuring collection_2
    let max_voter_weight_record = core_voter_test
        .get_max_voter_weight_record(&max_voter_weight_record_cookie.address)
        .await;
    assert_eq!(max_voter_weight_record.max_voter_weight_expiry, Some(0));

    // Update max voter weight record to make it valid again
    core_voter_test
        .update_max_voter_weight_record(&registrar_cookie, &mut max_voter_weight_record_cookie)
        .await?;

    // Fetch registrar account and assert that collection was added to the registrars collection_configs
    let registrar = core_voter_test
        .get_registrar_account(&registrar_cookie.address)
        .await;

    let max_voter_weight_record = core_voter_test
        .get_max_voter_weight_record(&max_voter_weight_record_cookie.address)
        .await;

    // Assert that max_voter_weight_expiry is Some(current_slot) after updating max voter weight record
    let max_voter_weight_total =
        (collection_1_weight * collection_1_size) + (collection_2_weight * collection_2_size);

    assert!(registrar.collection_configs.len() == 2);
    let current_slot = core_voter_test.bench.get_clock().await.slot;
    assert_eq!(max_voter_weight_record.max_voter_weight_expiry, Some(current_slot));
    assert!(max_voter_weight_record.max_voter_weight == max_voter_weight_total as u64);

    Ok(())
}
