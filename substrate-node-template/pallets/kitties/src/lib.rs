#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

use frame_support::traits::Currency;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod kitty;
pub use crate::kitty::Kitty;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use crate::{BalanceOf, Kitty};
	use codec::{Codec, MaxEncodedLen};
	use core::fmt::Debug;
	use frame_support::{
		pallet_prelude::{StorageMap, *},
		traits::{Randomness, ReservableCurrency},
		PalletId,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use scale_info::{prelude::vec::Vec, TypeInfo};
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, Bounded, Hash, One, Zero};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// all kitties Counter
	#[pallet::storage]
	#[pallet::getter(fn all_kittyies_count)]
	pub type AllKittiesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::type_value]
	pub fn GetDefaultValue<T: Config>() -> T::KittyIndex {
		T::KittyIndex::default()
	}

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittyId<T: Config> =
		StorageValue<_, T::KittyIndex, ValueQuery, GetDefaultValue<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty<T::Hash, BalanceOf<T>>>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;

	// key is account id
	// value is vector kitty index
	#[pallet::storage]
	#[pallet::getter(fn owner_kitties)]
	pub type OwnerKitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<T::KittyIndex, T::MaxKittyLen>>;

	// key is kitty_id
	// value is (sell account, escrow account)
	#[pallet::storage]
	#[pallet::getter(fn sell_kitties)]
	pub type SellKitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, (T::AccountId, T::AccountId)>;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// Kitty Index
		type KittyIndex: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen
			+ TypeInfo;

		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// Reservation fee.
		#[pallet::constant]
		type ReservationFee: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type MaxKittyLen: Get<u32>;

		#[pallet::constant]
		type EscrowAccount: Get<PalletId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// kitty created
		KittyCreated { create: T::AccountId, kitty_index: T::KittyIndex },
		/// kitty bred
		KittyBred { create: T::AccountId, kitty_index: T::KittyIndex },
		/// kitty transfer
		TransferKitty { from: T::AccountId, to: T::AccountId, kitty_index: T::KittyIndex },
		/// Sell kitty
		SellKitty { seller: T::AccountId, escrow: T::AccountId, kitty_index: T::KittyIndex },
		/// cancel sell kitty
		CancelSellKitty { escrow: T::AccountId, seller: T::AccountId, kitty_index: T::KittyIndex },
		/// Buy kitty
		BuyKitty { seller: T::AccountId, buyer: T::AccountId, kitty_index: T::KittyIndex },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// invalid kitty id
		InvalidKittyId,
		/// same kitty id
		SameKittyId,
		/// not owner
		NotOwner,
		/// Empty Kitties
		EmptyKitties,
		/// max length kitties
		MaxLenKitties,
		/// kitty no sell
		KittyNoSell,
		/// should not same kitty owner
		ShouldNotSame,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// create a kitty
		#[pallet::weight(10_000)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::MaxLenKitties)?;
			// dbg!(kitty_id);
			// let kitty_index_max = T::KittyIndex::max_value();
			// dbg!(kitty_index_max);

			let dna = Self::random_hash(&who);
			let kitty = Kitty::<T::Hash, BalanceOf<T>> {
				id: dna.clone(),
				dna: dna.clone(),
				price: BalanceOf::<T>::default(),
				gender: Kitty::<T, T>::gender(dna.clone()),
			};

			Self::store_kitty(kitty_id, kitty.clone(), &who)?;

			// deposit token to who
			let deposit = T::ReservationFee::get();
			T::Currency::reserve(&who, deposit)?;

			// emit event
			Self::deposit_event(Event::KittyCreated { create: who, kitty_index: kitty_id });

			Ok(().into())
		}

		/// breed a kitty
		#[pallet::weight(10_000)]
		pub fn breed(
			origin: OriginFor<T>,
			father: T::KittyIndex,
			mother: T::KittyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(father != mother, Error::<T>::SameKittyId);

			let kitty_1 = Self::get_kitty(father).map_err(|_| Error::<T>::InvalidKittyId)?;
			// dbg!(kitty_1.clone());
			let kitty_2 = Self::get_kitty(mother).map_err(|_| Error::<T>::InvalidKittyId)?;
			// dbg!(kitty_2.clone());

			// get next_id
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::MaxLenKitties)?;

			// select for breeding
			let selector = Self::random_hash(&who);
			let mut new_dna = T::Hash::default();
			for i in 0..new_dna.as_ref().len() {
				new_dna.as_mut()[i] = kitty_1.dna.as_ref()[i] & selector.as_ref()[i] |
					kitty_2.dna.as_ref()[i] & !selector.as_ref()[i];
			}

			let new_kitty = Kitty::<T::Hash, BalanceOf<T>> {
				id: new_dna.clone(),
				dna: new_dna.clone(),
				price: BalanceOf::<T>::default(),
				gender: Kitty::<T, T>::gender(new_dna.clone()),
			};

			Self::store_kitty(kitty_id, new_kitty.clone(), &who)?;

			// emit event
			Self::deposit_event(Event::KittyBred { creater: who, kitty_index: kitty_id });

			Ok(().into())
		}

		/// transfer kitty
		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::inner_transfer(&who, &new_owner, &kitty_id)
		}

		/// buy a kitty
		#[pallet::weight(10_000)]
		pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			// get kitty owner account and escrow account
			let (sell_kitty_owner, escrow_account) =
				Self::sell_kitties(kitty_id).ok_or(Error::<T>::KittyNoSell)?;
			ensure!(escrow_account == Self::account_id(), Error::<T>::KittyNoSell);
			ensure!(sell_kitty_owner != owner, Error::<T>::ShouldNotSame);

			<KittyOwner<T>>::insert(kitty_id, owner.clone());

			<SellKitties<T>>::remove(kitty_id);

			let deposit = T::ReservationFee::get();
			let err_amount = T::Currency::unreserve(&sell_kitty_owner, deposit);
			debug_assert!(err_amount.is_zero());

			// emit event
			Self::deposit_event(Event::BuyKitty {
				buyer: owner,
				seller: sell_kitty_owner,
				kitty_index: kitty_id,
			});
			Ok(().into())
		}

		/// seller a kitty
		#[pallet::weight(10_000)]
		pub fn sell(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
			// get owner
			let owner = ensure_signed(origin)?;

			ensure!(Self::kitty_owner(kitty_id) == Some(owner.clone()), Error::<T>::NotOwner);

			// escrow kitty account
			let escrow_account = Self::account_id();

			<KittyOwner<T>>::insert(kitty_id, escrow_account.clone());

			// append sell kitties queue
			<SellKitties<T>>::insert(kitty_id, (owner.clone(), escrow_account.clone()));

			// emit event
			Self::deposit_event(Event::SellKitty {
				seller: owner,
				escrow: escrow_account,
				kitty_index: kitty_id,
			});
			Ok(().into())
		}

		/// cancel sell a kitty
		#[pallet::weight(10_000)]
		pub fn cancel_sell(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			// ensure kitty_id have been set to Sell kitties queue
			// use sell_kitties function we get they are (owner account and escrow_account)
			ensure!(
				Self::sell_kitties(kitty_id) == Some((owner.clone(), Self::account_id())),
				Error::<T>::KittyNoSell
			);

			// set owner
			<KittyOwner<T>>::insert(kitty_id, owner.clone());

			// remove kitty from sell kitty queue
			<SellKitties<T>>::remove(kitty_id);

			// emit event
			Self::deposit_event(Event::CancelSellKitty {
				escrow: Self::account_id(),
				seller: owner,
				kitty_index: kitty_id,
			});
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		// escrow account
		fn account_id() -> T::AccountId {
			T::EscrowAccount::get().into_account_truncating()
		}

		// store kitty
		fn store_kitty(
			kitty_id: T::KittyIndex,
			kitty: Kitty<T::Hash, BalanceOf<T>>,
			who: &T::AccountId,
		) -> DispatchResult {
			// set kitty id, and kitty
			Kitties::<T>::insert(kitty_id, &kitty);
			// set kitty id and owner
			KittyOwner::<T>::insert(kitty_id, &who);
			// update kitty id
			NextKittyId::<T>::set(kitty_id + T::KittyIndex::one());

			// add kitty id to owner
			if OwnerKitties::<T>::contains_key(&who) {
				OwnerKitties::<T>::mutate(&who, |value| -> Result<(), sp_runtime::DispatchError> {
					if let Some(v) = value {
						v.try_push(kitty_id).map_err(|_| Error::<T>::MaxLenKitties)?;
					}
					Ok(())
				})?
			} else {
				let mut value: BoundedVec<T::KittyIndex, T::MaxKittyLen> =
					BoundedVec::with_bounded_capacity(10);
				value.force_push(kitty_id);
				OwnerKitties::<T>::insert(&who, &value);
			}

			Ok(().into())
		}

		// get a random_value
		fn random_hash(sender: &T::AccountId) -> T::Hash {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);

			T::Hashing::hash_of(&payload)
		}

		// get next identifier
		fn get_next_id() -> Result<T::KittyIndex, ()> {
			match Self::next_kitty_id() {
				val if val == T::KittyIndex::max_value() => Err(()),
				val => Ok(val),
			}
		}

		// get kitty by id
		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty<T::Hash, BalanceOf<T>>, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}

		// get all kitty by account owner
		pub fn get_all_kitties(
			owner: &T::AccountId,
		) -> Result<Vec<Kitty<T::Hash, BalanceOf<T>>>, sp_runtime::DispatchError> {
			let all_kitty_index = OwnerKitties::<T>::get(&owner).ok_or(Error::<T>::EmptyKitties)?;

			let mut result = Vec::new();
			for kitty_index in all_kitty_index.iter() {
				let kitty = Kitties::<T>::get(&kitty_index).ok_or(Error::<T>::EmptyKitties)?;
				result.push(kitty);
			}

			Ok(result)
		}

		pub fn inner_transfer(
			from: &T::AccountId,
			to: &T::AccountId,
			kitty_id: &T::KittyIndex,
		) -> DispatchResult {
			ensure!(Self::kitty_owner(kitty_id) == Some(from.clone()), Error::<T>::NotOwner);

			<KittyOwner<T>>::insert(kitty_id, to.clone());

			// emit event
			Self::deposit_event(Event::TransferKitty {
				from: from.clone(),
				to: to.clone(),
				kitty_index: kitty_id.clone(),
			});

			Ok(().into())
		}
	}
}
