use crate::{self as pallet_task_market, *};
use frame_support::{
    assert_ok, assert_noop, parameter_types,
    PalletId,
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
        TaskMarket: pallet_task_market,
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

parameter_types! {
    pub const TaskMarketPalletId: PalletId = PalletId(*b"taskmark");
    pub const MaxTitleLength: u32 = 128;
    pub const MaxDescriptionLength: u32 = 1024;
    pub const MaxProposalLength: u32 = 512;
    pub const MaxBidsPerTask: u32 = 20;
    pub const MinTaskReward: u64 = 100;
    pub const MaxActiveTasksPerAccount: u32 = 50;
}

impl pallet_task_market::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Currency = Balances;
    type ReputationManager = Reputation;
    type PalletId = TaskMarketPalletId;
    type MaxTitleLength = MaxTitleLength;
    type MaxDescriptionLength = MaxDescriptionLength;
    type MaxProposalLength = MaxProposalLength;
    type MaxBidsPerTask = MaxBidsPerTask;
    type MinTaskReward = MinTaskReward;
    type MaxActiveTasksPerAccount = MaxActiveTasksPerAccount;
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
fn post_task_works() {
    new_test_ext().execute_with(|| {
        let poster = 1;
        let title = b"Build a website".to_vec();
        let description = b"Need a React website".to_vec();
        let reward = 1000u64;
        let deadline = 1000u64;

        // Post task
        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(poster),
            title.clone(),
            description,
            reward,
            deadline
        ));

        // Check task was created
        let task = TaskMarket::tasks(0).unwrap();
        assert_eq!(task.poster, poster);
        assert_eq!(task.reward, reward);
        assert_eq!(task.status, TaskStatus::Open);

        // Check escrow was reserved
        assert_eq!(Balances::reserved_balance(poster), reward);

        // Check reputation stats updated
        let rep = Reputation::reputations(poster);
        assert_eq!(rep.total_tasks_posted, 1);
        assert_eq!(rep.total_spent, reward);
    });
}

#[test]
fn post_task_fails_if_reward_too_low() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TaskMarket::post_task(
                RuntimeOrigin::signed(1),
                b"Task".to_vec(),
                b"Description".to_vec(),
                50, // Below MinTaskReward (100)
                1000
            ),
            Error::<Test>::RewardTooLow
        );
    });
}

#[test]
fn bid_on_task_works() {
    new_test_ext().execute_with(|| {
        let poster = 1;
        let bidder = 2;

        // Post task
        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(poster),
            b"Task".to_vec(),
            b"Description".to_vec(),
            1000,
            1000
        ));

        // Submit bid
        assert_ok!(TaskMarket::bid_on_task(
            RuntimeOrigin::signed(bidder),
            0,
            800,
            b"I can do this".to_vec()
        ));

        // Check bid was stored
        let bid = TaskMarket::task_bids(0, bidder).unwrap();
        assert_eq!(bid.bidder, bidder);
        assert_eq!(bid.amount, 800);
    });
}

#[test]
fn cannot_bid_on_own_task() {
    new_test_ext().execute_with(|| {
        let poster = 1;

        // Post task
        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(poster),
            b"Task".to_vec(),
            b"Description".to_vec(),
            1000,
            1000
        ));

        // Try to bid on own task
        assert_noop!(
            TaskMarket::bid_on_task(
                RuntimeOrigin::signed(poster),
                0,
                800,
                b"Proposal".to_vec()
            ),
            Error::<Test>::CannotBidOnOwnTask
        );
    });
}

#[test]
fn assign_task_works() {
    new_test_ext().execute_with(|| {
        let poster = 1;
        let bidder = 2;

        // Post task
        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(poster),
            b"Task".to_vec(),
            b"Description".to_vec(),
            1000,
            1000
        ));

        // Submit bid
        assert_ok!(TaskMarket::bid_on_task(
            RuntimeOrigin::signed(bidder),
            0,
            800,
            b"Proposal".to_vec()
        ));

        // Assign task
        assert_ok!(TaskMarket::assign_task(
            RuntimeOrigin::signed(poster),
            0,
            bidder
        ));

        // Check task status
        let task = TaskMarket::tasks(0).unwrap();
        assert_eq!(task.status, TaskStatus::Assigned);
        assert_eq!(task.assigned_to, Some(bidder));
    });
}

#[test]
fn only_poster_can_assign() {
    new_test_ext().execute_with(|| {
        let poster = 1;
        let bidder = 2;
        let other = 3;

        // Post task and bid
        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(poster),
            b"Task".to_vec(),
            b"Description".to_vec(),
            1000,
            1000
        ));
        assert_ok!(TaskMarket::bid_on_task(
            RuntimeOrigin::signed(bidder),
            0,
            800,
            b"Proposal".to_vec()
        ));

        // Try to assign from non-poster
        assert_noop!(
            TaskMarket::assign_task(RuntimeOrigin::signed(other), 0, bidder),
            Error::<Test>::NotPoster
        );
    });
}

#[test]
fn submit_and_approve_work_releases_escrow() {
    new_test_ext().execute_with(|| {
        let poster = 1;
        let worker = 2;

        // Post, bid, assign
        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(poster),
            b"Task".to_vec(),
            b"Description".to_vec(),
            1000,
            1000
        ));
        assert_ok!(TaskMarket::bid_on_task(
            RuntimeOrigin::signed(worker),
            0,
            800,
            b"Proposal".to_vec()
        ));
        assert_ok!(TaskMarket::assign_task(
            RuntimeOrigin::signed(poster),
            0,
            worker
        ));

        let worker_balance_before = Balances::free_balance(worker);

        // Submit work
        assert_ok!(TaskMarket::submit_work(
            RuntimeOrigin::signed(worker),
            0,
            b"https://proof.com".to_vec()
        ));

        // Approve work
        assert_ok!(TaskMarket::approve_work(RuntimeOrigin::signed(poster), 0));

        // Check task status
        let task = TaskMarket::tasks(0).unwrap();
        assert_eq!(task.status, TaskStatus::Approved);

        // Check payment transferred
        assert_eq!(
            Balances::free_balance(worker),
            worker_balance_before + 1000
        );
        // Poster started with 10000, now has 9000 (paid 1000 to worker)
        assert_eq!(Balances::free_balance(poster), 9000);
        assert_eq!(Balances::reserved_balance(poster), 0);

        // Check reputation updated
        let rep = Reputation::reputations(worker);
        assert_eq!(rep.total_tasks_completed, 1);
        assert_eq!(rep.successful_completions, 1);
        assert_eq!(rep.total_earned, 1000);
    });
}

#[test]
fn cancel_task_refunds_escrow() {
    new_test_ext().execute_with(|| {
        let poster = 1;

        // Post task
        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(poster),
            b"Task".to_vec(),
            b"Description".to_vec(),
            1000,
            1000
        ));

        // Check escrow reserved
        assert_eq!(Balances::reserved_balance(poster), 1000);

        // Cancel task
        assert_ok!(TaskMarket::cancel_task(RuntimeOrigin::signed(poster), 0));

        // Check escrow released
        assert_eq!(Balances::reserved_balance(poster), 0);

        // Check task status
        let task = TaskMarket::tasks(0).unwrap();
        assert_eq!(task.status, TaskStatus::Cancelled);
    });
}

#[test]
fn cannot_cancel_assigned_task() {
    new_test_ext().execute_with(|| {
        let poster = 1;
        let worker = 2;

        // Post, bid, assign
        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(poster),
            b"Task".to_vec(),
            b"Description".to_vec(),
            1000,
            1000
        ));
        assert_ok!(TaskMarket::bid_on_task(
            RuntimeOrigin::signed(worker),
            0,
            800,
            b"Proposal".to_vec()
        ));
        assert_ok!(TaskMarket::assign_task(
            RuntimeOrigin::signed(poster),
            0,
            worker
        ));

        // Try to cancel
        assert_noop!(
            TaskMarket::cancel_task(RuntimeOrigin::signed(poster), 0),
            Error::<Test>::InvalidTaskStatus
        );
    });
}

#[test]
fn dispute_task_works() {
    new_test_ext().execute_with(|| {
        let poster = 1;
        let worker = 2;

        // Post, bid, assign, submit
        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(poster),
            b"Task".to_vec(),
            b"Description".to_vec(),
            1000,
            1000
        ));
        assert_ok!(TaskMarket::bid_on_task(
            RuntimeOrigin::signed(worker),
            0,
            800,
            b"Proposal".to_vec()
        ));
        assert_ok!(TaskMarket::assign_task(
            RuntimeOrigin::signed(poster),
            0,
            worker
        ));
        assert_ok!(TaskMarket::submit_work(
            RuntimeOrigin::signed(worker),
            0,
            b"Proof".to_vec()
        ));

        // Poster disputes
        assert_ok!(TaskMarket::dispute_task(
            RuntimeOrigin::signed(poster),
            0,
            b"Work is incomplete".to_vec()
        ));

        // Check status
        let task = TaskMarket::tasks(0).unwrap();
        assert_eq!(task.status, TaskStatus::Disputed);
    });
}

#[test]
fn resolve_dispute_updates_reputation() {
    new_test_ext().execute_with(|| {
        let poster = 1;
        let worker = 2;

        // Post, bid, assign, submit, dispute
        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(poster),
            b"Task".to_vec(),
            b"Description".to_vec(),
            1000,
            1000
        ));
        assert_ok!(TaskMarket::bid_on_task(
            RuntimeOrigin::signed(worker),
            0,
            800,
            b"Proposal".to_vec()
        ));
        assert_ok!(TaskMarket::assign_task(
            RuntimeOrigin::signed(poster),
            0,
            worker
        ));
        assert_ok!(TaskMarket::submit_work(
            RuntimeOrigin::signed(worker),
            0,
            b"Proof".to_vec()
        ));
        assert_ok!(TaskMarket::dispute_task(
            RuntimeOrigin::signed(poster),
            0,
            b"Dispute".to_vec()
        ));

        let poster_rep_before = Reputation::reputations(poster).score;
        let worker_rep_before = Reputation::reputations(worker).score;

        // Resolve dispute in favor of worker
        assert_ok!(TaskMarket::resolve_dispute(
            RuntimeOrigin::root(),
            0,
            worker
        ));

        // Check reputation changes
        // Worker wins: +200, Poster loses: -500
        assert_eq!(
            Reputation::reputations(worker).score,
            worker_rep_before + 200
        );
        assert_eq!(
            Reputation::reputations(poster).score,
            poster_rep_before - 500
        );

        // Check dispute stats
        assert_eq!(Reputation::reputations(worker).disputes_won, 1);
        assert_eq!(Reputation::reputations(poster).disputes_lost, 1);
    });
}

#[test]
fn task_count_increments() {
    new_test_ext().execute_with(|| {
        assert_eq!(TaskMarket::task_count(), 0);

        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(1),
            b"Task 1".to_vec(),
            b"Description".to_vec(),
            1000,
            1000
        ));
        assert_eq!(TaskMarket::task_count(), 1);

        assert_ok!(TaskMarket::post_task(
            RuntimeOrigin::signed(1),
            b"Task 2".to_vec(),
            b"Description".to_vec(),
            1000,
            1000
        ));
        assert_eq!(TaskMarket::task_count(), 2);
    });
}
