#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

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

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		// TODO Part II: The Type of Random we want to specify for runtime.
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
		// TODO Part III: helper functions for dispatchable functions

		// TODO: increment_nonce, random_hash, mint, transfer_from
	}
}
