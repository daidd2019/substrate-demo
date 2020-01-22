#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, dispatch::DispatchResult, ensure};
use system::ensure_signed;


pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as SimpleMapStorage {
		SimpleMap get(fn simple_map): map T::AccountId => u32;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		fn set_single_entry(origin, entry: u32) -> DispatchResult {
			let user = ensure_signed(origin)?;
			<SimpleMap<T>>::insert(user.clone(), entry);

			Self::deposit_event(RawEvent::EntrySet(user, entry));
			Ok(())
		}

		fn get_single_entry(origin, account: T::AccountId) -> DispatchResult {
			let getter = ensure_signed(origin)?;
			ensure!(<SimpleMap<T>>::exists(account.clone()), "an entry does not exist for this user");
			let entry = <SimpleMap<T>>::get(account);

			Self::deposit_event(RawEvent::EntryGot(getter, entry));
			Ok(())
		}

		fn take_single_entry(origin) -> DispatchResult {
			let taker = ensure_signed(origin)?;
			ensure!(<SimpleMap<T>>::exists(taker.clone()), "an entry does not exist for this user");
			let entry = <SimpleMap<T>>::take(taker.clone());

			Self::deposit_event(RawEvent::EntryTook(taker, entry));

			Ok(())
		}

		fn increase_single_entry(origin, add_this_val: u32) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let old_entry = <SimpleMap<T>>::get(&sender);
			let new_entry = old_entry.checked_add(add_this_val).ok_or("value overflowed")?;
			<SimpleMap<T>>::insert(sender, new_entry);

			Self::deposit_event(RawEvent::IncreaseEntry(old_entry, new_entry));
			Ok(())
		}

		fn compare_and_swap_single_entry(origin, old_entry: u32, new_entry: u32) -> DispatchResult{
			let user = ensure_signed(origin)?;
			ensure!(old_entry == <SimpleMap<T>>::get(user.clone()), "cas failed bc old_entry inputted by user != existing_entry");
			<SimpleMap<T>>::insert(user, new_entry);

			Self::deposit_event(RawEvent::CAS(old_entry, new_entry));
			Ok(())
		}

	}
}

decl_event!(
	pub enum Event<T> 
	where 
		AccountId = <T as system::Trait>::AccountId,
	{
		EntrySet(AccountId, u32),
		EntryGot(AccountId, u32),
		EntryTook(AccountId, u32),
		IncreaseEntry(u32, u32),
		CAS(u32, u32),
	}
);

#[cfg(test)]
mod tests {
	use super::{Module, Trait, RawEvent};

	use sp_core::H256;
	use frame_support::{impl_outer_origin, impl_outer_event, assert_ok, assert_err, parameter_types, weights::Weight};
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

	mod simple_map_event {
		pub use crate::simple_map::Event;
    }

    impl_outer_event! {
        pub enum TestEvent for TestRuntime {
            simple_map_event<T>,
        }
    }

	impl Trait for TestRuntime {
		type Event = TestEvent;
	}

	type System = system::Module<TestRuntime>;
	type SimpleMapModule = Module<TestRuntime>;

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
	fn set_entry_works() {
		ExtBuilder::build().execute_with(||{
			assert_ok!(SimpleMapModule::set_single_entry(Origin::signed(1), 1));
			let expected_event = TestEvent::simple_map_event(RawEvent::EntrySet(1, 1));
			assert!(System::events().iter().any(|a| a.event == expected_event));
		})
	}

	#[test]
	fn get_entry_err_works() {
		ExtBuilder::build().execute_with(||{
			assert_err!(SimpleMapModule::get_single_entry(Origin::signed(1), 2), "an entry does not exist for this user");
		})	
	}

	#[test]
	fn get_entry_works() {
		ExtBuilder::build().execute_with(||{
			assert_ok!(SimpleMapModule::set_single_entry(Origin::signed(1), 10));
			assert_ok!(SimpleMapModule::get_single_entry(Origin::signed(2), 1));

			let expected_event = TestEvent::simple_map_event(RawEvent::EntryGot(2, 10));
			assert!(System::events().iter().any(|a| a.event == expected_event));
		})
	}

	#[test]
	fn take_entry_err_works() {
		ExtBuilder::build().execute_with(||{
			assert_err!(SimpleMapModule::take_single_entry(Origin::signed(1)), "an entry does not exist for this user");
		})
	}

	#[test]
	fn take_entry_works() {
		ExtBuilder::build().execute_with(||{
			assert_ok!(SimpleMapModule::set_single_entry(Origin::signed(1), 10));
			assert_ok!(SimpleMapModule::take_single_entry(Origin::signed(1)));

			let expected_event = TestEvent::simple_map_event(RawEvent::EntryTook(1, 10));
			assert!(System::events().iter().any(|a| a.event == expected_event));
			//data not exist after take
			assert_err!(SimpleMapModule::get_single_entry(Origin::signed(2), 1), "an entry does not exist for this user");
		})
	}

	#[test]
	fn increase_works() {
		ExtBuilder::build().execute_with(||{
			assert_ok!(SimpleMapModule::set_single_entry(Origin::signed(1), 10));
			assert_ok!(SimpleMapModule::increase_single_entry(Origin::signed(1), 15));

			let expected_event = TestEvent::simple_map_event(RawEvent::IncreaseEntry(10, 25));
			assert!(System::events().iter().any(|a| a.event == expected_event));
		})
	}

	#[test]
	fn increase_no_exists_works() {
		ExtBuilder::build().execute_with(||{
			assert_ok!(SimpleMapModule::increase_single_entry(Origin::signed(1), 5));

			let expected_event = TestEvent::simple_map_event(RawEvent::IncreaseEntry(0, 5));
			assert!(System::events().iter().any(|a| a.event == expected_event));
		})
	}

	#[test]
	fn cas_works(){
		ExtBuilder::build().execute_with(||{
			assert_ok!(SimpleMapModule::set_single_entry(Origin::signed(1), 10));
			assert_err!(SimpleMapModule::compare_and_swap_single_entry(Origin::signed(1), 5, 20), "cas failed bc old_entry inputted by user != existing_entry");

			assert_ok!(SimpleMapModule::compare_and_swap_single_entry(Origin::signed(1), 10, 20));
			let expected_event = TestEvent::simple_map_event(RawEvent::CAS(10, 20));
			assert!(System::events().iter().any(|a| a.event == expected_event));
		})
	}

}
