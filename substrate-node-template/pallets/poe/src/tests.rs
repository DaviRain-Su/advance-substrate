use crate::{mock::*, Error, Config, Proofs};
use frame_support::{assert_noop, assert_ok, BoundedVec};


#[test]
fn create_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

        let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

        assert_eq!(Proofs::<Test>::get(&bounded_claim), Some((1, frame_system::Pallet::<Test>::block_number())));
    })
}

#[test]
fn create_claim_failed_when_already_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

        assert_noop!(PoeModule::create_claim(Origin::signed(1), claim.clone()), Error::<Test>::ProofAlreadyExist);
    })
}

#[test]
fn create_claim_failed_when_proof_tolong() {
    new_test_ext().execute_with(|| {
        let claim = vec![1; 513];

        assert_noop!(PoeModule::create_claim(Origin::signed(1), claim.clone()), Error::<Test>::ClaimTooLong);

    })
}

#[test]
fn revoke_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

        let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

        assert_eq!(Proofs::<Test>::get(&bounded_claim), Some((1, frame_system::Pallet::<Test>::block_number())));

        assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));

        assert_eq!(Proofs::<Test>::get(&bounded_claim), None);
    })
}

#[test]
fn revoke_claim_failed_when_claim_to_long() {
    new_test_ext().execute_with(|| {
        let claim = vec![1; 513];

        assert_noop!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()), Error::<Test>::ClaimTooLong);
    })
}

#[test]
fn revoke_claim_failed_when_claim_no_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_noop!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()), Error::<Test>::ClaimNotExist);
    })
}

#[test]
fn revoke_claim_failed_when_not_claim_owner() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

        let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

        assert_eq!(Proofs::<Test>::get(&bounded_claim), Some((1, frame_system::Pallet::<Test>::block_number())));

        assert_noop!(PoeModule::revoke_claim(Origin::signed(2), claim.clone()), Error::<Test>::NotClaimOwner);
    })
}

#[test]
fn transfer_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

        let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

        assert_eq!(Proofs::<Test>::get(&bounded_claim), Some((1, frame_system::Pallet::<Test>::block_number())));

        // transfer claim
        assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2));

        assert_eq!(Proofs::<Test>::get(&bounded_claim), Some((2, frame_system::Pallet::<Test>::block_number())));
    })
}

#[test]
fn transfer_claim_failed_when_claim_tolong() {
    new_test_ext().execute_with(|| {
        let claim1 = vec![1; 513];

        // transfer claim
        assert_noop!(PoeModule::transfer_claim(Origin::signed(1), claim1.clone(), 2), Error::<Test>::ClaimTooLong);
    })
}

#[test]
fn transfer_claim_failed_when_claim_no_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

        let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

        assert_eq!(Proofs::<Test>::get(&bounded_claim), Some((1, frame_system::Pallet::<Test>::block_number())));

        let claim1 = vec![1, 2];

        // transfer claim
        assert_noop!(PoeModule::transfer_claim(Origin::signed(1), claim1.clone(), 2), Error::<Test>::ClaimNotExist);
     })
}

#[test]
fn transfer_claim_failed_when_no_claim_owner() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

        let bounded_claim = BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();

        assert_eq!(Proofs::<Test>::get(&bounded_claim), Some((1, frame_system::Pallet::<Test>::block_number())));

        // transfer claim
        assert_noop!(PoeModule::transfer_claim(Origin::signed(2), claim.clone(), 3), Error::<Test>::NotClaimOwner);
     })
}