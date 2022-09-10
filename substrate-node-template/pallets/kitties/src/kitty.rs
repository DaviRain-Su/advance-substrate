use core::fmt::Debug;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{Parameter, RuntimeDebug};
use frame_support::traits::tokens::Balance;
use scale_info::TypeInfo;
use sp_runtime::traits::{
    CheckEqual, MaybeDisplay, MaybeMallocSizeOf, MaybeSerializeDeserialize,
	Member, SimpleBitOps,
};

// Kitty Information struct
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub struct Kitty<Hash, Balance> {
	id: Hash,
	dna: Hash,
	price: Balance,
	gender: Gender,
}

impl<Hash, Balances> Kitty<Hash, Balances>
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
	pub fn new(id: Hash) -> Self {
		Self { id: id.clone(), dna: id.clone(), price: 0u32.into(), gender: Gender::default() }
	}
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, Eq, PartialEq)]
pub enum Gender {
	Male,
	Female,
}

impl Default for Gender {
	fn default() -> Self {
		Gender::Male
	}
}

// TODO part II: Enum and implementation to handle Gender  Type in Kitty struct
