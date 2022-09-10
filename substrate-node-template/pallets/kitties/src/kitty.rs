use core::fmt::Debug;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{Parameter, RuntimeDebug};
use frame_support::traits::tokens::Balance;
use scale_info::TypeInfo;
use sp_runtime::traits::{
    CheckEqual, MaybeDisplay, MaybeMallocSizeOf, MaybeSerializeDeserialize,
	Member, SimpleBitOps,
};
use serde::{Serialize, Deserialize};

// Kitty Information struct
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq, MaxEncodedLen)]
pub struct Kitty<Hash, Balance, AccountId> {
	pub dna: Hash,
	pub price: Option<Balance>,
	pub gender: Gender,
	pub owner: AccountId,
}

impl<Hash, Balances, AccountId> Kitty<Hash, Balances, AccountId>
where
    // copy from frame_support
	Hash: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ SimpleBitOps
		+ Ord
		+ Default
		+ Copy
		+ CheckEqual
		+ sp_std::hash::Hash
		+ AsRef<[u8]>
		+ AsMut<[u8]>
		+ MaybeMallocSizeOf
		+ MaxEncodedLen,
    // copy from frame_support::traits::tokens
	Balances: Balance + MaybeSerializeDeserialize + Debug + MaxEncodedLen,
{
	pub fn new(dna: Hash, owner: AccountId) -> Self {
		Self { dna, price: None, gender: Gender::default(), owner }
	}
}

// Enum declaration for Gender.
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Gender {
	Male,
	Female,
}

// Implementation to handle Gender type in Kitty struct.
impl Default for Gender {
	fn default() -> Self {
		Gender::Male
	}
}

// TODO part II: Enum and implementation to handle Gender  Type in Kitty struct
