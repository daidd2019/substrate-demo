#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, dispatch::DispatchResult, ensure};
use system::ensure_signed;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as SingleValueStorage {
		MyValue : u32;
		MyAccount: T::AccountId;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		pub fn set_value(origin, value: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let now = <system::Module<T>>::block_number();
			<MyValue>::put(value);

			Self::deposit_event(RawEvent::ValueSet(value, now));
			Ok(())
		}

		pub fn get_value(origin) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			ensure!(<MyValue>::exists(), "value does not exists");
			let now = <system::Module<T>>::block_number();
			let val = <MyValue>::get();

			Self::deposit_event(RawEvent::ValueGet(val, now));
			Ok(())
		}

		pub fn set_account(origin, account_id: T::AccountId) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let now = <system::Module<T>>::block_number();
			<MyAccount<T>>::put(account_id.clone());

			Self::deposit_event(RawEvent::AccountSet(account_id, now));
			Ok(())
		}

		pub fn get_account(origin) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			ensure!(<MyAccount<T>>::exists(), "Account does not exists");
			let now = <system::Module<T>>::block_number();
			let who = <MyAccount<T>>::get();

			Self::deposit_event(RawEvent::AccountGet(who, now));
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> 
	where 
		AccountId = <T as system::Trait>::AccountId,
		BlockNumber = <T as system::Trait>::BlockNumber,
	{
		ValueSet(u32, BlockNumber),
		ValueGet(u32, BlockNumber),
		AccountSet(AccountId, BlockNumber),
		AccountGet(AccountId, BlockNumber),
	}
);

#[cfg(test)]
mod tests {
	use super::{Module, Trait};

	use sp_core::H256;
	use frame_support::{impl_outer_origin, impl_outer_event, assert_ok, parameter_types, weights::Weight};
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
		type Event = TestEvent; 
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
		type ModuleToIndex = ();
	}

	mod single_value {
		pub use crate::single_value::Event;
    }

    impl_outer_event! {
        pub enum TestEvent for TestRuntime {
            single_value<T>,
        }
    }

	impl Trait for TestRuntime {
		type Event = TestEvent;
	}

	type System = system::Module<TestRuntime>;
	type SingleValueModule = Module<TestRuntime>;

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
			System::set_block_number(2);
			assert_ok!(SingleValueModule::set_value(Origin::signed(1), 42));

		});
	}
}
