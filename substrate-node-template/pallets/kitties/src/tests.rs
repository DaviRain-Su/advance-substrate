use crate::{mock::*, Error, Event as KittyEvent, MyKittiyIndex};
use frame_support::{assert_noop, assert_ok, PalletId};
use sp_runtime::{traits::AccountIdConversion, AccountId32};

#[test]
fn test_create_kitty_success() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		// let free_balance = Balances::free_balance(alice.clone());
		// let minimum_balance = <Balances as fungible::Inspect<AccountId32>>::minimum_balance();
		// println!("alice:free_balance = {}", free_balance);
		// println!("minimum_balance = {}", minimum_balance);

		assert_ok!(Kitties::create(Origin::signed(alice.clone())));

		assert_events(vec![Event::Kitties(KittyEvent::KittyCreated {
			creater: alice,
			kitty_index: MyKittiyIndex(0),
		})]);
	})
}

#[test]
fn test_create_kitty_faild() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		for _i in 0..512 {
			assert_ok!(Kitties::create(Origin::signed(alice.clone())));
		}

		assert_noop!(Kitties::create(Origin::signed(alice.clone())), Error::<Test>::MaxLenKitties);
	})
}

#[test]
fn test_breed_successfully() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		assert_ok!(Kitties::create(Origin::signed(alice.clone()))); // kitty_id 0
		assert_ok!(Kitties::create(Origin::signed(alice.clone()))); // kitty_id 1

		assert_ok!(Kitties::breed(
			Origin::signed(alice.clone()),
			MyKittiyIndex(0),
			MyKittiyIndex(1)
		)); // kitty_id 2

		assert_events(vec![Event::Kitties(KittyEvent::KittyBred {
			creater: alice,
			kitty_index: MyKittiyIndex(2),
		})]);
	})
}

#[test]
fn test_breed_failed_by_same_kitty_id() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		assert_ok!(Kitties::create(Origin::signed(alice.clone()))); // kitty_id 0
		assert_ok!(Kitties::create(Origin::signed(alice.clone()))); // kitty_id 1

		assert_noop!(
			Kitties::breed(Origin::signed(alice.clone()), MyKittiyIndex(0), MyKittiyIndex(0)),
			Error::<Test>::SameKittyId
		); // kitty_id 2
	})
}

#[test]
fn test_breed_failed_not_create_father_and_mother() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		assert_noop!(
			Kitties::breed(Origin::signed(alice.clone()), MyKittiyIndex(0), MyKittiyIndex(1)),
			Error::<Test>::InvalidKittyId
		); // kitty_id 2
	})
}

#[test]
fn test_breed_failed_create_max_len_kitties() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		for _i in 0..512 {
			assert_ok!(Kitties::create(Origin::signed(alice.clone())));
		}

		assert_noop!(
			Kitties::breed(Origin::signed(alice.clone()), MyKittiyIndex(510), MyKittiyIndex(511)),
			Error::<Test>::MaxLenKitties
		); // kitty_id 2
	})
}

#[test]
fn test_transfer_success() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		let bob: AccountId32 = PalletId(*b"py/bob00").into_account_truncating();

		assert_ok!(Kitties::create(Origin::signed(alice.clone())));

		assert_ok!(Kitties::transfer(Origin::signed(alice.clone()), MyKittiyIndex(0), bob.clone()));

		assert_events(vec![Event::Kitties(KittyEvent::TransferKitty {
			from: alice,
			to: bob,
			kitty_index: MyKittiyIndex(0),
		})]);
	})
}
#[test]
fn test_transfer_failed_no_owner() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		let bob: AccountId32 = PalletId(*b"py/bob00").into_account_truncating();

		assert_ok!(Kitties::create(Origin::signed(alice.clone())));

		assert_noop!(
			Kitties::transfer(Origin::signed(alice.clone()), MyKittiyIndex(1), bob.clone()),
			Error::<Test>::NotOwner
		);
	})
}

#[test]
fn test_sell_kitty_success() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		assert_ok!(Kitties::create(Origin::signed(alice.clone())));

		assert_ok!(Kitties::sell(Origin::signed(alice.clone()), MyKittiyIndex(0)));

		let escrow_account: AccountId32 = PalletId(*b"py/kitti").into_account_truncating();

		assert_events(vec![Event::Kitties(KittyEvent::SellKitty {
			seller: alice,
			escrow: escrow_account,
			kitty_index: MyKittiyIndex(0),
		})]);
	})
}

#[test]
fn test_sell_kitty_failed_no_owner() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		assert_ok!(Kitties::create(Origin::signed(alice.clone())));

		assert_noop!(
			Kitties::sell(Origin::signed(alice.clone()), MyKittiyIndex(1)),
			Error::<Test>::NotOwner
		);
	})
}

#[test]
fn test_cancel_sell_success() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		assert_ok!(Kitties::create(Origin::signed(alice.clone())));

		assert_ok!(Kitties::sell(Origin::signed(alice.clone()), MyKittiyIndex(0)));

		assert_ok!(Kitties::cancel_sell(Origin::signed(alice.clone()), MyKittiyIndex(0)));

		let escrow_account: AccountId32 = PalletId(*b"py/kitti").into_account_truncating();

		assert_events(vec![Event::Kitties(KittyEvent::CancelSellKitty {
			escrow: escrow_account,
			seller: alice,
			kitty_index: MyKittiyIndex(0),
		})]);
	})
}

#[test]
fn test_cancel_sell_failed_no_owner() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		assert_ok!(Kitties::create(Origin::signed(alice.clone())));

		assert_ok!(Kitties::sell(Origin::signed(alice.clone()), MyKittiyIndex(0)));

		assert_noop!(
			Kitties::cancel_sell(Origin::signed(alice.clone()), MyKittiyIndex(1)),
			Error::<Test>::KittyNoSell
		);
	})
}

#[test]
fn test_buy_kitty_success() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();
		let bob: AccountId32 = PalletId(*b"py/bob00").into_account_truncating();

		assert_ok!(Kitties::create(Origin::signed(alice.clone())));

		assert_ok!(Kitties::sell(Origin::signed(alice.clone()), MyKittiyIndex(0)));

		assert_ok!(Kitties::buy(Origin::signed(bob.clone()), MyKittiyIndex(0)));

		assert_events(vec![Event::Kitties(KittyEvent::BuyKitty {
			buyer: bob,
			seller: alice,
			kitty_index: MyKittiyIndex(0),
		})]);
	})
}

#[test]
fn test_buy_kitty_faild_kitty_no_sell() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();
		let bob: AccountId32 = PalletId(*b"py/bob00").into_account_truncating();

		assert_ok!(Kitties::create(Origin::signed(alice.clone())));

		assert_ok!(Kitties::sell(Origin::signed(alice.clone()), MyKittiyIndex(0)));

		assert_noop!(
			Kitties::buy(Origin::signed(bob.clone()), MyKittiyIndex(1)),
			Error::<Test>::KittyNoSell
		);
	})
}

#[test]
fn test_buy_kitty_faild_should_no_same_owner() {
	new_test_ext().execute_with(|| {
		let alice: AccountId32 = PalletId(*b"py/alice").into_account_truncating();

		assert_ok!(Kitties::create(Origin::signed(alice.clone())));

		assert_ok!(Kitties::sell(Origin::signed(alice.clone()), MyKittiyIndex(0)));

		assert_noop!(
			Kitties::buy(Origin::signed(alice.clone()), MyKittiyIndex(0)),
			Error::<Test>::ShouldNotSame
		);
	})
}
