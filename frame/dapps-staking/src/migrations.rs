//! Dapps staking migration utility module

use super::*;

pub mod v2 {

    use super::*;
    use frame_support::{traits::Get, weights::Weight};

    #[cfg(feature = "try-runtime")]
    use frame_support::log;
    #[cfg(feature = "try-runtime")]
    use frame_support::traits::OnRuntimeUpgradeHelpersExt;
    #[cfg(feature = "try-runtime")]
    use sp_runtime::traits::Zero;

    // The old value used to store locked amount
    type OldLedger<T> = pallet::pallet::BalanceOf<T>;

    #[cfg(feature = "try-runtime")]
    pub fn pre_migrate<T: Config, U: OnRuntimeUpgradeHelpersExt>() -> Result<(), &'static str> {
        assert_eq!(Version::V1_0_0, StorageVersion::<T>::get());

        let ledger_count = Ledger::<T>::iter_keys().count() as u64;
        U::set_temp_storage::<u64>(ledger_count, "ledger_count");

        log::info!(">>> PreMigrate: ledger count: {:?}", ledger_count,);

        Ok(().into())
    }

    pub fn migrate<T: Config>() -> Weight {
        if StorageVersion::<T>::get() != Version::V1_0_0 {
            return T::DbWeight::get().reads(1);
        }

        let ledger_size = Ledger::<T>::iter_keys().count() as u64;

        Ledger::<T>::translate(|_, value: OldLedger<T>| {
            Some(AccountLedger {
                locked: value,
                unbonding_info: Default::default(),
            })
        });

        StorageVersion::<T>::put(Version::V2_0_0);

        T::DbWeight::get().reads_writes(ledger_size, ledger_size + 1)
    }

    #[cfg(feature = "try-runtime")]
    pub fn post_migrate<T: Config, U: OnRuntimeUpgradeHelpersExt>() -> Result<(), &'static str> {
        assert_eq!(Version::V2_0_0, StorageVersion::<T>::get());

        let init_ledger_count = U::get_temp_storage::<u64>("ledger_count").unwrap();
        let current_ledger_count = Ledger::<T>::iter_keys().count() as u64;
        assert_eq!(init_ledger_count, current_ledger_count);

        for acc_ledger in Ledger::<T>::iter_values() {
            assert!(acc_ledger.locked > Zero::zero());
            assert!(acc_ledger.unbonding_info.is_empty());
        }

        log::info!(">>> PostMigrate: ledger count: {:?}", current_ledger_count,);

        Ok(())
    }
}
