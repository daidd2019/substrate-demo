#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, dispatch::DispatchResult, ensure,
	StorageMap, StorageLinkedMap, StorageValue};
use system::ensure_signed;


pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as SimpleMapStorage {
		TheList get(fn the_list): map u32 => T::AccountId;
		TheCounter get(fn the_counter): u32;

		LinkedList get(fn linked_list): linked_map u32 => T::AccountId;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
		fn add_member(origin, account: T::AccountId)  -> DispatchResult{
			let _ = ensure_signed(origin)?;

			let now_counter = <TheCounter>::get();
			let next_counter = now_counter.checked_add(1).ok_or("value overflowed")?;

			<TheCounter>::put(next_counter);
			<TheList<T>>::insert(next_counter, account.clone());

			let linked_head = match  <LinkedList<T>>::head() {
				Some(head) => head,
				None => 0,
			};

			<LinkedList<T>>::insert(linked_head + 1, account.clone());

			Self::deposit_event(RawEvent::MemberAdded(account, next_counter));
			Ok(())
		}

		fn remove_member_bunded(origin, index: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;

            ensure!(<TheList<T>>::exists(index), "an element doesn't exist at this index");

            let largest_index = <TheCounter>::get();
            let member_to_remove = <TheList<T>>::take(index);
            // swap
            if index != largest_index {
                let temp = <TheList<T>>::take(largest_index);
                <TheList<T>>::insert(index, temp);
          //      <TheList<T>>::insert(largest_index, member_to_remove.clone());
            }
            // pop
         //   <TheList<T>>::remove(largest_index);
            <TheCounter>::put(largest_index - 1);

            Self::deposit_event(RawEvent::MemberRemoved(member_to_remove, index));
            Ok(())
		}

		fn remove_member_linked(origin, index: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			ensure!(<LinkedList<T>>::exists(index), "A member does not exist at this index");
			let head = <LinkedList<T>>::head().unwrap();

			let member_to_remove = <LinkedList<T>>::take(index);
			if index != head {
				let head_member = <LinkedList<T>>::take(head); 
				<LinkedList<T>>::insert(index, head_member);

			}
			
			Self::deposit_event(RawEvent::MemberRemoved(member_to_remove, index));

			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> 
	where 
		AccountId = <T as system::Trait>::AccountId,
	{
		MemberAdded(AccountId, u32),
		MemberRemoved(AccountId, u32),
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

	mod linked_map_event {
		pub use crate::linked_map::Event;
    }

    impl_outer_event! {
        pub enum TestEvent for TestRuntime {
            linked_map_event<T>,
        }
    }

	impl Trait for TestRuntime {
		type Event = TestEvent;
	}

	type System = system::Module<TestRuntime>;
	type LinkedMapModule = Module<TestRuntime>;

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
	fn add_member_works() {
		ExtBuilder::build().execute_with(||{
			assert_ok!(LinkedMapModule::add_member(Origin::signed(1), 1));
			let expected_event = TestEvent::linked_map_event(RawEvent::MemberAdded(1, 1));
			assert!(System::events().iter().any(|a| a.event == expected_event));

			assert_ok!(LinkedMapModule::add_member(Origin::signed(1), 2));
			let expected_event = TestEvent::linked_map_event(RawEvent::MemberAdded(2, 2));
			assert!(System::events().iter().any(|a| a.event == expected_event));
		})
	}

	#[test]
	fn remove_member_works() {
		ExtBuilder::build().execute_with(||{

			assert_err!(LinkedMapModule::remove_member_bunded(Origin::signed(1), 4), "an element doesn't exist at this index");
			assert_ok!(LinkedMapModule::add_member(Origin::signed(1), 5));
			assert_ok!(LinkedMapModule::add_member(Origin::signed(1), 2));
			assert_ok!(LinkedMapModule::add_member(Origin::signed(1), 6));

			assert_ok!(LinkedMapModule::remove_member_bunded(Origin::signed(1), 3));
			let expected_event = TestEvent::linked_map_event(RawEvent::MemberRemoved(6, 3));
			assert!(System::events().iter().any(|a| a.event == expected_event));

			assert_ok!(LinkedMapModule::remove_member_bunded(Origin::signed(1), 1));
			let expected_event = TestEvent::linked_map_event(RawEvent::MemberRemoved(5, 1));
			assert!(System::events().iter().any(|a| a.event == expected_event));

		})
	}

	#[test]
	fn remove_member_linked_works() {
		ExtBuilder::build().execute_with(||{

			assert_ok!(LinkedMapModule::add_member(Origin::signed(1), 1));
			assert_ok!(LinkedMapModule::add_member(Origin::signed(1), 2));
			assert_ok!(LinkedMapModule::add_member(Origin::signed(1), 3));
			assert_ok!(LinkedMapModule::add_member(Origin::signed(1), 4));
			assert_ok!(LinkedMapModule::add_member(Origin::signed(1), 5));
			assert_ok!(LinkedMapModule::add_member(Origin::signed(1), 6));

			assert_ok!(LinkedMapModule::remove_member_linked(Origin::signed(1), 3));
			let expected_event = TestEvent::linked_map_event(RawEvent::MemberRemoved(3, 3));
			assert!(System::events().iter().any(|a| a.event == expected_event));

			assert_ok!(LinkedMapModule::remove_member_linked(Origin::signed(1), 3));
			let expected_event = TestEvent::linked_map_event(RawEvent::MemberRemoved(6, 3));
			assert!(System::events().iter().any(|a| a.event == expected_event));

		})
	}
}
