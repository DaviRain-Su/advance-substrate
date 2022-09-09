#![cfg_attr(not(feature = "std"), no_std)]

use core::ops::Add;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Currency;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use scale_info::TypeInfo;

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

pub trait KittiyConstant<T: Config> {
	fn one() -> Self;

	fn max() -> Self;
}

#[derive(Decode, Encode, Debug, PartialEq, Eq, TypeInfo, Copy, Clone, MaxEncodedLen, Default)]
pub struct MyKittiyIndex(u32);

impl<T: Config> KittiyConstant<T> for MyKittiyIndex {
	fn one() -> Self {
		MyKittiyIndex(1)
	}

	fn max() -> Self {
		MyKittiyIndex(u32::MAX)
	}
}

impl Add for MyKittiyIndex {
	type Output = MyKittiyIndex;

	fn add(self, rhs: Self) -> Self::Output {
		MyKittiyIndex(self.0 + rhs.0)
	}
}

#[frame_support::pallet]
pub mod pallet {
	use crate::{BalanceOf, KittiyConstant, Kitty};
	use codec::{Encode, EncodeLike, MaxEncodedLen};
	use core::{fmt::Debug, ops::Add};
	use frame_support::{
		pallet_prelude::{StorageMap, *},
		traits::{Currency, Randomness, ReservableCurrency},
		PalletId,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use scale_info::{prelude::vec::Vec, TypeInfo};
	use sp_runtime::traits::{AccountIdConversion, CheckedConversion, Hash};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// all kitties Counter
	#[pallet::storage]
	#[pallet::getter(fn all_kittyies_count)]
	pub type AllKittiesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::type_value]
	pub fn GetDefaultValue<T: Config>() -> T::KittiyIndex {
		T::KittiyIndex::default()
	}

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittityId<T: Config> =
		StorageValue<_, T::KittiyIndex, ValueQuery, GetDefaultValue<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittiyIndex, Kitty<T::Hash, BalanceOf<T>>>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittiyIndex, T::AccountId>;

	// key is account id
	// value is vector kitty index
	#[pallet::storage]
	#[pallet::getter(fn owner_kitties)]
	pub type OwnerKitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<T::KittiyIndex, T::MaxKittyLen>>;

	// key is kitty_id
	// value is (sell account, escrow account)
	#[pallet::storage]
	#[pallet::getter(fn sell_kitties)]
	pub type SellKitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittiyIndex, (T::AccountId, T::AccountId)>;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		type KittiyIndex: TypeInfo
			+ Decode
			+ Encode
			+ PartialEq
			+ Eq
			+ Clone
			+ Debug
			+ EncodeLike
			+ MaxEncodedLen
			+ Default
			+ Add<Output = Self::KittiyIndex>
			+ KittiyConstant<Self>
			+ Copy;

		type Currency: Currency<Self::AccountId>;

		type ReservableCurrency: ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type MaxKittyLen: Get<u32>;

		#[pallet::constant]
		type EscrowAccount: Get<PalletId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// kitty created
		KittyCreated(T::AccountId, T::KittiyIndex, Kitty<T::Hash, BalanceOf<T>>),
		/// kitty bred
		KittyBred(T::AccountId, T::KittiyIndex, Kitty<T::Hash, BalanceOf<T>>),
		/// kitty transfer
		TransferKitty(T::AccountId, T::AccountId, T::KittiyIndex),
		/// Sell kitty
		SellKitty(T::AccountId, T::AccountId, T::KittiyIndex),
		/// cancel sell kitty
		CancelSellKitty(T::AccountId, T::AccountId, T::KittiyIndex),
		/// Buy kitty
		BuyKitty(T::AccountId, T::AccountId, T::KittiyIndex),
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
		///
		MaxLenKitties,
		///
		KittNoSell,
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
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			let dna = Self::random_hash(&who);
			let kitty = Kitty::<T::Hash, BalanceOf<T>> {
				id: dna.clone(),
				dna: dna.clone(),
				price: BalanceOf::<T>::default(),
				gender: Kitty::<T, T>::gender(dna.clone()),
			};

			Self::store_kitty(kitty_id, kitty.clone(), &who)?;

			T::ReservableCurrency::reserve(&who, 1000_000_000_000_000_000_000u128.checked_into().unwrap())?;

			// emit event
			Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));

			Ok(().into())
		}

		/// breed a kitty
		#[pallet::weight(10_000)]
		pub fn breed(
			origin: OriginFor<T>,
			father: T::KittiyIndex,
			mother: T::KittiyIndex,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(father != mother, Error::<T>::SameKittyId);

			let kitty_1 = Self::get_kitty(father).map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::get_kitty(mother).map_err(|_| Error::<T>::InvalidKittyId)?;

			// get next_id
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

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
			Self::deposit_event(Event::KittyBred(who, kitty_id, new_kitty));

			Ok(().into())
		}

		/// transfer kitty
		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			kitty_id: T::KittiyIndex,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::inner_transfer(&who, &new_owner, &kitty_id)
		}

		/// buy a kitty
		#[pallet::weight(10_000)]
		pub fn buy(origin: OriginFor<T>, kitty_id: T::KittiyIndex) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			// get kitty owner account and escrow account
			let (self_kitty_owner, escorw_account) =
				Self::sell_kitties(kitty_id).ok_or(Error::<T>::KittNoSell)?;
			ensure!(escorw_account != Self::account_id(), Error::<T>::KittNoSell);

			<KittyOwner<T>>::insert(kitty_id, owner.clone());

			<SellKitties<T>>::remove(kitty_id);

			// emit event
			Self::deposit_event(Event::BuyKitty(owner, self_kitty_owner, kitty_id));
			Ok(().into())
		}

		/// seller a kitty
		#[pallet::weight(10_000)]
		pub fn sell(origin: OriginFor<T>, kitty_id: T::KittiyIndex) -> DispatchResult {
			// get owner
			let owner = ensure_signed(origin)?;

			ensure!(Self::kitty_owner(kitty_id) == Some(owner.clone()), Error::<T>::NotOwner);

			// escrow kitty account
			let escrow_account = Self::account_id();

			<KittyOwner<T>>::insert(kitty_id, escrow_account.clone());

			// append sell kitties queue
			<SellKitties<T>>::insert(kitty_id, (owner.clone(), escrow_account.clone()));

			// emit event
			Self::deposit_event(Event::SellKitty(owner, escrow_account, kitty_id));
			Ok(().into())
		}

		/// cancel sell a kitty
		#[pallet::weight(10_000)]
		pub fn cancel_sell(origin: OriginFor<T>, kitty_id: T::KittiyIndex) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			ensure!(
				Self::sell_kitties(kitty_id) != Some((owner.clone(), Self::account_id())),
				Error::<T>::KittNoSell
			);

			// set owner
			<KittyOwner<T>>::insert(kitty_id, owner.clone());

			// remove kitty from sell kitty queue
			<SellKitties<T>>::remove(kitty_id);

			// emit event
			Self::deposit_event(Event::CancelSellKitty(Self::account_id(), owner, kitty_id));
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
			kitty_id: T::KittiyIndex,
			kitty: Kitty<T::Hash, BalanceOf<T>>,
			who: &T::AccountId,
		) -> DispatchResult {
			// set kitty id, and kitty
			Kitties::<T>::insert(kitty_id, &kitty);
			// set kitty id and owner
			KittyOwner::<T>::insert(kitty_id, &who);
			// update kitty id
			NextKittityId::<T>::set(kitty_id.add(T::KittiyIndex::one()));

			// add kittyid to owner
			if OwnerKitties::<T>::contains_key(&who) {
				OwnerKitties::<T>::mutate(&who, |value| -> Result<(), sp_runtime::DispatchError> {
					if let Some(v) = value {
						v.try_push(kitty_id).map_err(|_| Error::<T>::MaxLenKitties)?;
					}
					Ok(())
				})?
			} else {
				let mut value: BoundedVec<T::KittiyIndex, T::MaxKittyLen> =
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

			// payload.using_encoded(blake2_128)
			T::Hashing::hash_of(&payload)
		}

		// get next identifier
		fn get_next_id() -> Result<T::KittiyIndex, ()> {
			match Self::next_kitty_id() {
				val if val == T::KittiyIndex::max() => Err(()),
				val => Ok(val),
			}
		}

		// get kitty by id
		fn get_kitty(kitty_id: T::KittiyIndex) -> Result<Kitty<T::Hash, BalanceOf<T>>, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}

		// get all kitty by account owner
		pub fn get_all_kitties(
			owner: &T::AccountId,
		) -> Result<Vec<Kitty<T::Hash, BalanceOf<T>>>, sp_runtime::DispatchError> {
			let all_kitty_indexs =
				OwnerKitties::<T>::get(&owner).ok_or(Error::<T>::EmptyKitties)?;

			let mut result = Vec::new();
			for kitty_index in all_kitty_indexs.iter() {
				let kitty = Kitties::<T>::get(&kitty_index).ok_or(Error::<T>::EmptyKitties)?;
				result.push(kitty);
			}

			Ok(result)
		}

		pub fn inner_transfer(
			from: &T::AccountId,
			to: &T::AccountId,
			kitty_id: &T::KittiyIndex,
		) -> DispatchResult {
			ensure!(Self::kitty_owner(kitty_id) == Some(from.clone()), Error::<T>::NotOwner);

			<KittyOwner<T>>::insert(kitty_id, to.clone());

			// emit event
			Self::deposit_event(Event::TransferKitty(from.clone(), to.clone(), kitty_id.clone()));

			Ok(().into())
		}
	}
}
