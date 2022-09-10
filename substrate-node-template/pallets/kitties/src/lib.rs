#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

use frame_support::traits::Currency;
// Re-export pallet module information
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
// About Kitty information
mod kitty;
// Re-export kitty information
pub use kitty::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Randomness;
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Hash;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		// The Type of Random we want to specify for runtime.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// TODO Part II
	}

	// Error
	#[pallet::error]
	pub enum Error<T> {
		// TODO Part II
	}

	// define AllKittiesCount to count Kitties.
	// keep track of a counter that will correspond to the total amount of Kitties in existence.
	// 追踪已经产出的kitty的总的数量
	// [substrate runtime storage](https://docs.substrate.io/build/runtime-storage/)
	#[pallet::storage]
	#[pallet::getter(fn all_kitties_count)]
	pub(super) type AllKittiesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_nonce)]
	pub(super) type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	// TODO Part II: Remaining storage item.

	// TODO Part III: Our pallet's generic configuration.

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO part III: create_kitty

		// TODO Part III: set_price

		// TODO Part III: transfer

		// TODO Part III: buy_kitty

		// TODO Part III: breed_kitty
	}

	// TODO Parts II: helper function for Kitty struct

	impl<T: Config> Pallet<T> {
		pub fn gender(dna: T::Hash) -> Gender {
			if dna.as_ref()[0] % 2 == 0 {
				Gender::Male
			} else {
				Gender::Female
			}
		}

		fn increment_nonce() -> DispatchResult {
			Nonce::<T>::try_mutate(|nonce| {
				let next = nonce.checked_add(1).expect("Overflow"); // TODO: Add Error handle
				*nonce = next;

				Ok(().into())
			})
		}

		// takes in an AccountId and returns the hash of a random seed, AccountId and the current nonce.
		fn random_hash(sender: &T::AccountId) -> T::Hash {
			let nonce = Self::get_nonce();
			let seed = T::KittyRandomness::random_seed();

			T::Hashing::hash_of(&(seed, &sender, nonce))
		}
		// TODO Part III: helper functions for dispatchable functions

		// TODO: increment_nonce, random_hash, mint, transfer_from
	}
}
