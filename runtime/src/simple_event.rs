#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, dispatch::DispatchResult};
use system::ensure_signed;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as SimpleEventStorage {
		Value get(fn value): Option<u32>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		pub fn set_value(origin, value: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Value::put(value);

			Self::deposit_event(RawEvent::EmitInput(value, who));
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		EmitInput(u32, AccountId),
	}
);

#[cfg(test)]
mod tests {
	use super::{Module, Trait};

	use sp_core::H256;
	use frame_support::{impl_outer_origin, assert_ok, parameter_types, weights::Weight};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
	};

	impl_outer_origin! {
		pub enum Origin for TestRuntime {}
	}

	#[derive(Clone, Eq, PartialEq)]
	pub struct TestRuntime;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for TestRuntime {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
		type ModuleToIndex = ();
	}
	impl Trait for TestRuntime {
		type Event = ();
	}
	type SimpleEventModule = Module<TestRuntime>;

	pub struct ExtBuilder;
	impl ExtBuilder {
        pub fn build() -> sp_io::TestExternalities {
            let storage = system::GenesisConfig::default()
                .build_storage::<TestRuntime>()
                .unwrap();
            sp_io::TestExternalities::from(storage)
        }
    }

	#[test]
	fn it_works_for_default_value() {
		ExtBuilder::build().execute_with(|| {
			assert_ok!(SimpleEventModule::set_value(Origin::signed(1), 42));
			assert_eq!(SimpleEventModule::value(), Some(42));
		});
	}
}
