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

		/// The Currency handler for the Kitties pallet.
		type Currency: Currency<Self::AccountId>;

		// The Type of Random we want to specify for runtime.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// The maximum amount of Kitties a single account can own.
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// TODO Part II
	}

	// Error
	#[pallet::error]
	pub enum Error<T> {
		// kitty not exist
		KittyNotExist,
	}

	#[pallet::storage]
	#[pallet::getter(fn all_kitties_count)]
	/// Keeps track of the number of Kitties in existence.
	/// 记录已经产生的kitty数量
	pub(super) type AllKittiesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	/// Stores a Kitty's unique traits, owner and price.
	/// 根据kitty id 键值对映射 kitty
	pub(super) type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Kitty<T::Hash, BalanceOf<T>, T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	/// Keeps track of what accounts own what Kitty.
	/// 追踪一个账户拥有多少kitty
	pub(super) type KittiesOwned<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<T::Hash, T::MaxKittyOwned>, ValueQuery>;

	// TODO Part II: Remaining storage item.

	// TODO Part III: Our pallet's generic configuration.

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new unique kitty.
		///
		/// The actual creation is done in the `mint()` function
		/// 创建kitty
		#[pallet::weight(1000)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			//1. create_kitty
			//2. deposit `created` event

			Ok(().into())
		}

		// TODO Part III: set_price

		// TODO Part III: transfer

		// TODO Part III: buy_kitty

		// TODO Part III: breed_kitty
	}

	// Helper function for Kitty struct
	impl<T: Config> Pallet<T> {
		// 根据dna 随机分配gender
		pub fn gender(dna: T::Hash) -> Gender {
			match dna.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}
	}

	//** Our helper functions.**//
	impl<T: Config> Pallet<T> {
		// Generate a random gender value
		fn gen_gender() -> Gender {
			let random = T::KittyRandomness::random(&b"gender"[..]).0;
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}

		// Create new DNA with existing DNA
		// 根据两个kitty 孵化出一个新的kitty
		pub fn breed_dna(kid1: &T::Hash, kid2: &T::Hash) -> Result<T::Hash, Error<T>> {
			let dna1: T::Hash = Self::kitties(kid1).ok_or(Error::<T>::KittyNotExist)?.dna;
			let dna2: T::Hash  = Self::kitties(kid2).ok_or(Error::<T>::KittyNotExist)?.dna;

			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.as_ref().len() {
				new_dna.as_mut()[i] = (new_dna.as_ref()[i] & dna1.as_ref()[i]) | (!new_dna.as_ref()[i] & dna2.as_ref()[i]);
			}
			Ok(new_dna)
		}

		// Generate a random DNA value
		fn gen_dna() -> T::Hash {
			let payload = (
				T::KittyRandomness::random(&b"dna"[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);

			T::Hashing::hash_of(&payload)
		}

		// generate a random DNA value by AccountId
		fn gen_dna_by_account_id(sender: &T::AccountId) -> T::Hash {
			let seed = T::KittyRandomness::random_seed();
			let nonce = <frame_system::Pallet<T>>::block_number();

			T::Hashing::hash_of(&(seed, &sender, nonce))
		}

		// help to check correct kitty owner
		// 根据kitty_id 检索出kitty 判断是否与传入的account 相等
		pub fn is_kitty_owner(kitty_id: &T::Hash, acct: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *acct),
				None => Err(Error::<T>::KittyNotExist)
			}
		}
		// TODO Part III: helper functions for dispatchable functions

		// TODO: increment_nonce, random_hash, mint, transfer_from
	}
}
