use crate::*;
use codec::{Encode, Decode, MaxEncodedLen};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

// struct for holding kitty information
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Kitty<Hash, Balance> {
    pub id: Hash,
    pub dna: Hash, 
    pub price: Balance,
    pub gender: Gender,
}


impl<T: Config> Kitty<T, T> {
    pub fn gender(dna: T::Hash) -> Gender {
        if dna.as_ref()[0] % 2 == 0 {
            Gender::Male
        } else { 
            Gender::Female
        }
    }
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum Gender {
    Male,
    Female,
}

impl Default for Gender {
    fn default() -> Self {
        Gender::Male
    }
}