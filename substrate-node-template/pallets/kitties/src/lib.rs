#![cfg_attr(not(feature = "std"), no_std)]

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

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::{*, StorageMap};
	use frame_system::{pallet_prelude::*, ensure_signed};
	use scale_info::TypeInfo;
	use frame_support::traits::Randomness;
	use codec::Encode;
	use sp_io::hashing::blake2_128;

	/// kitty index
	type KittiyIndex = u32;

	#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct Kitty(pub [u8; 16]);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::type_value]
	pub fn GetDefaultValue() -> KittiyIndex {
		0
	}

	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)]
	pub type NextKittityId<T> = StorageValue<_, KittiyIndex, ValueQuery, GetDefaultValue>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)] 
	pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittiyIndex, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)] 
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittiyIndex, T::AccountId>;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		
		type Randomness: Randomness<Self::Hash,Self::BlockNumber> ;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// kitty created
		KittyCreated(T::AccountId, KittiyIndex, Kitty),
		/// kitty bred
		KittyBred(T::AccountId, KittiyIndex, Kitty),
		/// kitty transfer
		TransferKitty(T::AccountId, T::AccountId, KittiyIndex,  Kitty),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// invalid kitty id
		InvalidKittyId,
		/// same kitty id
		SameKittyId,
		// not owner 
		NotOwner,
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
			
			let dna = Self::random_value(&who); 
			let kitty = Kitty(dna);

			// set kitty id, and kitty
			Kitties::<T>::insert(kitty_id, &kitty);
			// set kitty id and owner
			KittyOwner::<T>::insert(kitty_id, &who); 
			// update kitty id 
			NextKittityId::<T>::set(kitty_id + 1);

			// emit event
			Self::deposit_event(Event::KittyCreated(who, kitty_id, kitty));

			Ok(().into())
		}

		/// breed a kitty
		#[pallet::weight(10_000)]
		pub fn breed(origin: OriginFor<T>, father: KittiyIndex, mother: KittiyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			ensure!(father != mother, Error::<T>::SameKittyId);

			let kitty_1 = Self::get_kitty(father).map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::get_kitty(mother).map_err(|_| Error::<T>::InvalidKittyId)?;

			// get next_id  
			let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;

			// select for breeding 
			let selector = Self::random_value(&who);
			let mut data = [0u8; 16];
			for i in 0..data.len() {
				data[i] = kitty_1.0[i] & selector[i] | kitty_2.0[i] & !selector[i];
			}

			let new_kitty = Kitty(data);

			// set kitty id, and kitty
			Kitties::<T>::insert(kitty_id, &new_kitty);
			// set kitty id and owner
			KittyOwner::<T>::insert(kitty_id, &who); 
			// update kitty id 
			NextKittityId::<T>::set(kitty_id + 1);

			// emit event
			Self::deposit_event(Event::KittyBred(who, kitty_id, new_kitty));

			Ok(().into())
		}

		/// transfer kitty 
		#[pallet::weight(10_000)]
		pub fn transfer(origin: OriginFor<T>, kitty_id: KittiyIndex, new_owner: T::AccountId) -> DispatchResult {
			let who =  ensure_signed(origin)?;

			let kitty = Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;

			ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()), Error::<T>::NotOwner);

			<KittyOwner<T>>::insert(kitty_id, new_owner.clone());

			// emit event
			Self::deposit_event(Event::TransferKitty(who, new_owner, kitty_id, kitty));
			
			Ok(().into())
		}
		
	}

	impl<T: Config> Pallet<T> {
		// get a random_value
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet::<T>>::extrinsic_index(),
			);

			payload.using_encoded(blake2_128)
		}

		// get next identifier
		fn get_next_id() -> Result<KittiyIndex, ()> {
			match Self::next_kitty_id() {
				KittiyIndex::MAX => Err(()),
				val => Ok(val),
			}
		}

		// get kitty by id
		fn get_kitty(kitty_id: KittiyIndex) -> Result<Kitty, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}

	}
}
