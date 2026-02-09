use crate::{self as pallet_reputation, *};
use frame_support::{
    assert_ok, assert_noop, parameter_types,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Reputation: pallet_reputation,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type RuntimeTask = ();
    type ExtensionsWeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = u64;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

parameter_types! {
    pub const MaxCommentLength: u32 = 256;
    pub const InitialReputation: u32 = 5000;
    pub const MaxReputationDelta: u32 = 500;
    pub const MaxHistoryLength: u32 = 100;
}

impl pallet_reputation::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Currency = Balances;
    type MaxCommentLength = MaxCommentLength;
    type InitialReputation = InitialReputation;
    type MaxReputationDelta = MaxReputationDelta;
    type MaxHistoryLength = MaxHistoryLength;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 10000), (2, 10000), (3, 10000)],
        dev_accounts: Default::default(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}

#[test]
fn initial_reputation_is_correct() {
    new_test_ext().execute_with(|| {
        // Check that new accounts start with initial reputation
        let rep = Reputation::reputations(1);
        assert_eq!(rep.score, 5000);
        assert_eq!(rep.total_tasks_completed, 0);
        assert_eq!(rep.total_tasks_posted, 0);
    });
}

#[test]
fn submit_review_works() {
    new_test_ext().execute_with(|| {
        let reviewer = 1;
        let reviewee = 2;
        let rating = 5;
        let comment = b"Excellent work!".to_vec();
        let task_id = 1;

        // Submit review
        assert_ok!(Reputation::submit_review(
            RuntimeOrigin::signed(reviewer),
            reviewee,
            rating,
            comment.clone(),
            task_id
        ));

        // Check review was stored
        let review = Reputation::reviews(reviewer, reviewee).unwrap();
        assert_eq!(review.rating, rating);
        assert_eq!(review.task_id, task_id);

        // Check reputation increased (5 stars = +500)
        let rep = Reputation::reputations(reviewee);
        assert_eq!(rep.score, 5500); // 5000 + 500
    });
}

#[test]
fn cannot_review_self() {
    new_test_ext().execute_with(|| {
        let account = 1;
        
        assert_noop!(
            Reputation::submit_review(
                RuntimeOrigin::signed(account),
                account,
                5,
                b"Self review".to_vec(),
                1
            ),
            Error::<Test>::SelfReview
        );
    });
}

#[test]
fn invalid_rating_fails() {
    new_test_ext().execute_with(|| {
        // Rating 0 should fail
        assert_noop!(
            Reputation::submit_review(
                RuntimeOrigin::signed(1),
                2,
                0,
                b"Comment".to_vec(),
                1
            ),
            Error::<Test>::InvalidRating
        );

        // Rating 6 should fail
        assert_noop!(
            Reputation::submit_review(
                RuntimeOrigin::signed(1),
                2,
                6,
                b"Comment".to_vec(),
                1
            ),
            Error::<Test>::InvalidRating
        );
    });
}

#[test]
fn reputation_clamped_at_max() {
    new_test_ext().execute_with(|| {
        let account = 1;

        // Submit multiple 5-star reviews to push over 10000
        for i in 0..25 {
            assert_ok!(Reputation::submit_review(
                RuntimeOrigin::signed(2),
                account,
                5,
                b"Great!".to_vec(),
                i
            ));
        }

        // Should be clamped at 10000
        let rep = Reputation::reputations(account);
        assert_eq!(rep.score, 10000);
    });
}

#[test]
fn slash_reputation_works() {
    new_test_ext().execute_with(|| {
        let account = 1;
        let slash_amount = 1000;
        let reason = b"Misbehavior detected".to_vec();

        // Initial reputation is 5000
        assert_eq!(Reputation::reputations(account).score, 5000);

        // Slash reputation (requires root)
        assert_ok!(Reputation::slash_reputation(
            RuntimeOrigin::root(),
            account,
            slash_amount,
            reason
        ));

        // Check reputation decreased
        let rep = Reputation::reputations(account);
        assert_eq!(rep.score, 4000); // 5000 - 1000
    });
}

#[test]
fn slash_reputation_requires_root() {
    new_test_ext().execute_with(|| {
        // Non-root should fail
        assert_noop!(
            Reputation::slash_reputation(
                RuntimeOrigin::signed(1),
                2,
                1000,
                b"Reason".to_vec()
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn reputation_manager_trait_works() {
    new_test_ext().execute_with(|| {
        let worker = 1;
        let poster = 2;
        let earned = 1000u64;
        let spent = 1000u64;

        // Test on_task_completed
        Reputation::on_task_completed(&worker, earned);
        let rep = Reputation::reputations(worker);
        assert_eq!(rep.total_tasks_completed, 1);
        assert_eq!(rep.successful_completions, 1);
        assert_eq!(rep.total_earned, earned);

        // Test on_task_posted
        Reputation::on_task_posted(&poster, spent);
        let rep = Reputation::reputations(poster);
        assert_eq!(rep.total_tasks_posted, 1);
        assert_eq!(rep.total_spent, spent);

        // Test get_reputation
        let score = Reputation::get_reputation(&worker);
        assert_eq!(score, 5000);

        // Test meets_minimum_reputation
        assert!(Reputation::meets_minimum_reputation(&worker, 4000));
        assert!(!Reputation::meets_minimum_reputation(&worker, 6000));
    });
}

#[test]
fn dispute_resolution_updates_reputation() {
    new_test_ext().execute_with(|| {
        let winner = 1;
        let loser = 2;

        // Initial scores
        assert_eq!(Reputation::reputations(winner).score, 5000);
        assert_eq!(Reputation::reputations(loser).score, 5000);

        // Resolve dispute
        Reputation::on_dispute_resolved(&winner, &loser);

        // Winner gains +200, loser loses -500
        assert_eq!(Reputation::reputations(winner).score, 5200);
        assert_eq!(Reputation::reputations(loser).score, 4500);

        // Check stats
        assert_eq!(Reputation::reputations(winner).disputes_won, 1);
        assert_eq!(Reputation::reputations(loser).disputes_lost, 1);
    });
}

#[test]
fn rating_scales_reputation_boost() {
    new_test_ext().execute_with(|| {
        let reviewee1 = 1;
        let reviewee2 = 2;
        let reviewee3 = 3;

        // 1-star review: +100
        assert_ok!(Reputation::submit_review(
            RuntimeOrigin::signed(10),
            reviewee1,
            1,
            b"Poor".to_vec(),
            1
        ));
        assert_eq!(Reputation::reputations(reviewee1).score, 5100);

        // 3-star review: +300
        assert_ok!(Reputation::submit_review(
            RuntimeOrigin::signed(10),
            reviewee2,
            3,
            b"Average".to_vec(),
            2
        ));
        assert_eq!(Reputation::reputations(reviewee2).score, 5300);

        // 5-star review: +500
        assert_ok!(Reputation::submit_review(
            RuntimeOrigin::signed(10),
            reviewee3,
            5,
            b"Excellent".to_vec(),
            3
        ));
        assert_eq!(Reputation::reputations(reviewee3).score, 5500);
    });
}
