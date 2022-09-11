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
	use frame_support::{
		pallet_prelude::*,
		traits::{ExistenceRequirement, Randomness},
		transactional,
	};
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
		/// 一个账户拥有的最多kitty的数量限制
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new kitty  was successful created. \[sender, kitty_id\]
		Created(T::AccountId, T::Hash),
		/// Kitty Price was successful set. \[sender, kitty_id, new_price\]
		PriceSet(T::AccountId, T::Hash, Option<BalanceOf<T>>),
		/// A kitty was successful transferred. \[from, to, kitty_id\]
		Transferred(T::AccountId, T::AccountId, T::Hash),
		/// A kitty was successfully bought. \[buyer, seller, kitty_id, bid_price\]
		Bought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),
		/// A new kitty  was successful breeded. \[sender, kitty_id\]
		Breeded(T::AccountId, T::Hash),
	}

	// Error
	#[pallet::error]
	pub enum Error<T> {
		/// Handle arithmetic overflow when incrementing the Kitty counter.
		KittyCounterOverflow,
		/// An AccountId cannot own more Kitties than `MaxKittyCount`.
		ExceedMaxKittyOwned,
		/// Buy cannot be owner.
		BuyerIsKittyOwner,
		/// Cannot transfer a kitty to its owner.
		TransferToSelf,
		/// Handles checking whether the Kitty exists.
		KittyNotExist,
		/// Handles checking that the kitty is owned by the account transferring, buying or setting
		/// a price for it
		NoKittyOwner,
		/// Ensure the kitty is for sale
		KittyNotForSale,
		/// Ensure that the buying price is greater than the asking price.
		KittyBidPriceToLow,
		/// Ensures that an account has enough funds to purchase a kitty.
		NotEnoughBalance,
		/// Kitty id already exists.
		KittyIdExists,
	}

	// Store item

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	/// Stores a Kitty's unique traits, owner and price.
	/// 将 kitty id 映射到 kitty
	pub(super) type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, Kitty<T::Hash, BalanceOf<T>, T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	/// kitty id 映射到 kitty的所有者(account)
	pub(super) type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_of_owner)]
	/// Keeps track of what accounts own what Kitty.
	/// 追踪一个账户拥有多少kitty
	pub(super) type OwnedKitty<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::MaxKittyOwned>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn all_kitties_array)]
	///`AllKittiesArray` which is a `map` from `u64` to `T::Hash`, add a getter function for this
	pub(super) type AllKittiesArray<T: Config> = StorageMap<_, Blake2_128Concat, u64, T::Hash>; // update u64 with kittyIndex

	#[pallet::storage]
	#[pallet::getter(fn all_kitties_count)]
	/// Keeps track of the number of Kitties in existence.
	/// 记录已经产生的kitty数量
	pub(super) type AllKittiesCount<T: Config> = StorageValue<_, u64, ValueQuery>; // update u64 with kittyIndex

	#[pallet::storage]
	#[pallet::getter(fn all_kitties_index)]
	pub(super) type AllKittiesIndex<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, u64>; // update u64 with kittyIndex

	#[pallet::storage]
	#[pallet::getter(fn kitty_of_owner_by_index)]
	pub(super) type OwnedKittiesArray<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::AccountId, u64), T::Hash>;

	#[pallet::storage]
	#[pallet::getter(fn owned_kitty_count)]
	pub(super) type OwnedKittiesCount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

	#[pallet::storage]
	pub(super) type OwnedKittiesIndex<T: Config> =
		StorageMap<_, Blake2_128Concat, T::Hash, u64, ValueQuery>;

	// Our pallet's genesis configuration
	// 创世配置数据
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, T::Hash, Gender)>,
	}

	// Required to implement default for GenesisConfig
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (acct, dna, gender) in &self.kitties {
				let _ = <Pallet<T>>::mint(acct, Some(dna.clone()), Some(gender.clone()));
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new unique kitty.
		///
		/// The actual creation is done in the `mint()` function
		/// 创建kitty
		#[pallet::weight(1000)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// mint一个新的kitty
			let kitty_id = Self::mint(&sender, None, None)?;

			// log to the consle
			log::info!("[create_kitty]: kitty_id = {:?}", kitty_id);

			// deposit `Created` Event
			// 触发事件
			Self::deposit_event(Event::Created(sender, kitty_id));

			Ok(().into())
		}

		/// Set the price for a kitty.
		///
		/// Update Kitty price and update storage
		#[pallet::weight(1000)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Ensure the kitty exists and is called by the kitty owner.
			// 确保kitty的kitty存在已经，kitty的所有者是调用方
			ensure!(Self::is_kitty_owner(&kitty_id, &sender)?, Error::<T>::NoKittyOwner);

			// one way to change price.
			// change kitty_id kitty price
			// 设置kitty的价格
			Kitties::<T>::mutate(&kitty_id, |kitty| {
				if let Some(k) = kitty {
					k.price = new_price;
				}
			});

			// // another way:
			// let mut kitty = Self::kitties(kitty_id).ok_or(Error::<T>::KittyIdExists)?;// this can
			// use unwrap because aprove have checked. kitty.price = new_price;
			// Kitties::<T>::insert(kitty_id, kitty);

			// deposit PriceSet event
			// 触发事件
			Self::deposit_event(Event::PriceSet(sender, kitty_id, new_price));

			Ok(().into())
		}

		/// Directly transfer a kitty to another recipient.
		///
		/// An account that holds a kitty can send it to another Account. This will reset the asking
		/// price of the kitty, marking it not for sale.
		#[pallet::weight(1000)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: T::Hash,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			// ensure the kitty exists and is called by the kitty owner.
			// 确保kitty的kitty存在已经kitty的所有者是调用方
			ensure!(Self::is_kitty_owner(&kitty_id, &from)?, Error::<T>::NoKittyOwner);

			// verify the kitty is not transferring back to its owner.
			// 检查发送方和接收方不能相等
			ensure!(from != to, Error::<T>::TransferToSelf);

			// verify the recipient has the capacity to receive one more kitty
			// 确保给账户放入kitty不会超过kitty的最大容量
			let to_owned = OwnedKitty::<T>::get(&to);
			ensure!(
				(to_owned.len() as u32) < T::MaxKittyOwned::get(),
				Error::<T>::ExceedMaxKittyOwned
			);

			// 转移kitty给接收方
			Self::transfer_kitty_to(&kitty_id, &to)?;

			// 触发事件
			Self::deposit_event(Event::Transferred(from, to, kitty_id));

			Ok(().into())
		}

		/// Buy a saleable kitty. The bid price provided from the buyer has to be equal or higher
		/// than the ask price from the seller.
		///
		/// This will reset the asking price of the kitty, marking it not for sale.
		/// Marking this method `transactional` so when an error is returned, we ensure no storage
		/// is changed.
		#[transactional]
		#[pallet::weight(100)]
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			bid_price: BalanceOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			// check the kitty exists and buyer is not the current kitty owner
			// 检查kitty是否存在以及购买者不能是kitty所有者自己
			let kitty = Self::kitties(&kitty_id).ok_or(Error::<T>::KittyNotExist)?;
			ensure!(kitty.owner != buyer, Error::<T>::BuyerIsKittyOwner);

			// check the kitty is for sale and the kitty ask price <= bid_price
			// 确保bid的价格大于或者等于kitty的标记的价格
			if let Some(ask_price) = kitty.price {
				ensure!(ask_price <= bid_price, Error::<T>::KittyBidPriceToLow);
			} else {
				Err(Error::<T>::KittyNotForSale)?;
			}

			// check the buyer has enough free balance
			// 确保买方有足够的余额购买kitty
			ensure!(T::Currency::free_balance(&buyer) >= bid_price, Error::<T>::NotEnoughBalance);

			// verify  the buyer has the capacity to receive once more kitty
			// 确保给账户放入kitty不会超过kitty的最大容量
			let to_owned = OwnedKitty::<T>::get(&buyer);
			ensure!(
				(to_owned.len() as u32) < T::MaxKittyOwned::get(),
				Error::<T>::ExceedMaxKittyOwned
			);

			// 从kitty中获得这个kitty的所有者
			let seller = kitty.owner.clone();

			// transfer the amount from buyer to seller
			// 转移bid的价格（native token）从买家发送给卖家
			T::Currency::transfer(&buyer, &seller, bid_price, ExistenceRequirement::KeepAlive)?;

			// transfer the kitty from seller to buyer
			// 转移kitty从卖方发送给买方
			Self::transfer_kitty_to(&kitty_id, &buyer)?;

			// 触发事件
			Self::deposit_event(Event::Bought(buyer, seller, kitty_id, bid_price));

			Ok(().into())
		}

		/// Breed a kitty.
		///
		/// Breed two kitties to create a new generation of Kitties.
		#[pallet::weight(1000)]
		pub fn breed_kitty(origin: OriginFor<T>, kid1: T::Hash, kid2: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// check verify `sender` owns both kitties (and both kitties exist).
			ensure!(Self::is_kitty_owner(&kid1, &sender)?, Error::<T>::NoKittyOwner); // 判断调用者是否有kid1这个kitty
			ensure!(Self::is_kitty_owner(&kid2, &sender)?, Error::<T>::NoKittyOwner); // 判断调用者是否有kid2这个kitty

			// 根据kid1 和kid2产生一个新的dna
			let new_dna = Self::breed_dna(&kid1, &kid2)?;

			// mint出一个新额kitty
			let kitty_id = Self::mint(&sender, Some(new_dna), None)?;

			// 触发事件
			Self::deposit_event(Event::Breeded(sender, kitty_id));

			Ok(().into())
		}
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
			let dna2: T::Hash = Self::kitties(kid2).ok_or(Error::<T>::KittyNotExist)?.dna;

			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.as_ref().len() {
				new_dna.as_mut()[i] = (new_dna.as_ref()[i] & dna1.as_ref()[i]) |
					(!new_dna.as_ref()[i] & dna2.as_ref()[i]);
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
				Some(kitty) => Ok(kitty.owner == *acct), /* 获取到kitty中的owner与传入的account对比 */
				None => Err(Error::<T>::KittyNotExist),
			}
		}

		// Helper to mint a Kitty.
		fn mint(
			owner: &T::AccountId,
			dna: Option<T::Hash>,
			gender: Option<Gender>,
		) -> Result<T::Hash, Error<T>> {
			// 根据传入的参数，owner， dna， gender 构造一个kitty
			let kitty = Kitty::<T::Hash, BalanceOf<T>, T::AccountId> {
				dna: dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender: gender.unwrap_or_else(Self::gen_gender),
				owner: owner.clone(),
			};

			// 计算出kitty_id
			let kitty_id = T::Hashing::hash_of(&kitty);
			// 确保新生成的kitty id在Kitties中没有存储
			ensure!(!Kitties::<T>::contains_key(kitty_id), Error::<T>::KittyIdExists);

			let owned_kitty_count: u64 = Self::owned_kitty_count(owner.clone());

			let new_owned_kitty_count =
				owned_kitty_count.checked_add(1).ok_or(Error::<T>::KittyCounterOverflow)?;

			let all_kitties_count: u64 = Self::all_kitties_count();

			let new_all_kitties_count =
				all_kitties_count.checked_add(1).ok_or(Error::<T>::KittyCounterOverflow)?;

			// Performs this operation first because as it may fail
			// 将这个kitty 放入到对应的账户下去
			OwnedKitty::<T>::try_mutate(&owner, |kitty_vec| kitty_vec.try_push(kitty_id))
				.map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;

			// 将kittyID和映射的accountID保存到KittyOwner
			KittyOwner::<T>::insert(kitty_id, owner.clone());

			// 将kitty_id和kitty 映射保存
			Kitties::<T>::insert(kitty_id, kitty);

			AllKittiesArray::<T>::insert(all_kitties_count, kitty_id.clone());
			// Performs this operation first as item fail
			// 将统计所有kitty 的计数器加一
			AllKittiesCount::<T>::put(new_all_kitties_count);
			AllKittiesIndex::<T>::insert(kitty_id, all_kitties_count);

			OwnedKittiesArray::<T>::insert((owner.clone(), owned_kitty_count), kitty_id.clone());
			OwnedKittiesCount::<T>::insert(owner.clone(), new_owned_kitty_count);
			OwnedKittiesIndex::<T>::insert(kitty_id.clone(), owned_kitty_count);

			Ok(kitty_id)
		}

		pub fn transfer_kitty_to(kitty_id: &T::Hash, to: &T::AccountId) -> DispatchResult {
			// 根据kitty_id获得 kitty
			let mut kitty = Self::kitties(&kitty_id).ok_or(Error::<T>::KittyNotExist)?;

			// 获取kitty中的所有者
			let pre_owner = kitty.owner.clone();

			let owned_kitty_count_from: u64 = Self::owned_kitty_count(&pre_owner);
			let owned_kitty_count_to: u64 = Self::owned_kitty_count(&to);

			let new_owned_kitty_count_from =
				owned_kitty_count_from.checked_add(1).ok_or(Error::<T>::KittyCounterOverflow)?;
			let new_owned_kitty_count_to =
				owned_kitty_count_to.checked_add(1).ok_or(Error::<T>::KittyCounterOverflow)?;

			let kitty_index = OwnedKittiesIndex::<T>::get(kitty_id);
			if kitty_index != new_owned_kitty_count_from {
				let last_kitty_id =
					<OwnedKittiesArray<T>>::get((pre_owner.clone(), new_owned_kitty_count_from))
						.ok_or(Error::<T>::KittyIdExists)?;
				<OwnedKittiesArray<T>>::insert((pre_owner.clone(), kitty_index), last_kitty_id);
				<OwnedKittiesIndex<T>>::insert(last_kitty_id, kitty_index);
			}

			OwnedKitty::<T>::try_mutate(&pre_owner, |owned| {
				if let Some(id) = owned.iter().position(|&id| id == *kitty_id) {
					owned.swap_remove(id);
					return Ok(())
				}
				Err(())
			})
			.map_err(|_| Error::<T>::KittyNotExist)?;

			// update the kitty owner
			// 更新kitty的所有者
			kitty.owner = to.clone();

			// reset the ask price so the kitty is not for sale until `set_price()`
			//  by the current owner.
			// 将kitty的价格清空
			kitty.price = None;

			// 更新kitty_id 对应的kitty的值
			Kitties::<T>::insert(kitty_id, kitty);

			// 更新kittyID和映射的accountID
			KittyOwner::<T>::insert(kitty_id, to.clone());
			<OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count_to);

			<OwnedKittiesArray<T>>::remove((pre_owner.clone(), new_owned_kitty_count_from));
			<OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count_to), kitty_id);

			<OwnedKittiesCount<T>>::insert(&pre_owner, new_owned_kitty_count_from);
			<OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count_to);

			// 转移之后将这个kitty添加到转移后的账户(to)中去
			OwnedKitty::<T>::try_mutate(&to, |vec| vec.try_push(*kitty_id))
				.map_err(|_| Error::<T>::ExceedMaxKittyOwned)?;

			Ok(())
		}
	}
}
