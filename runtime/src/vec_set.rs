#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, dispatch::DispatchResult, ensure};
use system::ensure_signed;
use sp_std::{vec, vec::Vec};


pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as VecMapStorage {
		Members get(fn members): Vec<T::AccountId>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		pub fn add_member(origin) -> DispatchResult {
			let member = ensure_signed(origin)?;
			ensure!(!Self::is_member(&member), "must not be a member to be added");
			<Members<T>>::append(&mut vec![member.clone()])?;

			Self::deposit_event(RawEvent::MemberAdded(member));
			Ok(())
		}

		pub fn remove_member(origin) -> DispatchResult {
			let member = ensure_signed(origin)?;
			ensure!(Self::is_member(&member), "must be a member in order to leave");
            <Members<T>>::mutate(|v| v.retain(|i| i != &member));

			Self::deposit_event(RawEvent::MemberRemoved(member));
			Ok(())
		}
	}
}

impl <T: Trait> Module<T> {
	pub fn is_member(who: &T::AccountId) -> bool {
		<Members<T>>::get().contains(who)
	}
}

decl_event!(
	pub enum Event<T> 
	where 
		AccountId = <T as system::Trait>::AccountId,
	{
		MemberAdded(AccountId),
		MemberRemoved(AccountId),
	}
);

#[cfg(test)]
mod tests {
	use super::{Module, Trait, RawEvent};

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

	mod vec_value_event {
		pub use crate::vec_set::Event;
    }

    impl_outer_event! {
        pub enum TestEvent for TestRuntime {
            vec_value_event<T>,
        }
    }

	impl Trait for TestRuntime {
		type Event = TestEvent;
	}

	type System = system::Module<TestRuntime>;
	type VecValueModule = Module<TestRuntime>;

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
	fn it_works_for_single_value() {
		ExtBuilder::build().execute_with(|| {

			let sender = Origin::signed(1);

			assert_ok!(VecValueModule::add_member(sender.clone()));
			assert_ok!(VecValueModule::remove_member(sender.clone()));

			let expected_event = TestEvent::vec_value_event(
				RawEvent::MemberAdded(1),
			);
			assert!(System::events().iter().any(|a| a.event == expected_event));
			let expected_event = TestEvent::vec_value_event(
				RawEvent::MemberRemoved(1),
			);
			assert!(System::events().iter().any(|a| a.event == expected_event));

		});
	}

}
