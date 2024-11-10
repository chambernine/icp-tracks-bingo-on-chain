use candid::{CandidType, Principal};
use ic_cdk::{query, update};
use ic_cdk_timers::{set_timer_interval, TimerId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use serde::Deserialize;
use ic_cdk::api::management_canister::main::raw_rand;
// use serde_json::{self};

const REQUIRED_PLAYERS: usize = 2;
const RANDOM_NUMBER_TIME_SECS: u64 = 15;
const CARD_SIZE: usize = 5;

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Card {
    numbers: Vec<Vec<u32>>,
    owner: Principal,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct GameState {
    cards: HashMap<Principal, Card>,
    called_numbers: HashSet<u32>,
    is_active: bool,
    winners: Vec<Principal>,
}

thread_local! {
    static STATE: RefCell<GameState> = RefCell::new(GameState {
        cards: HashMap::new(),
        called_numbers: HashSet::new(),
        is_active: false,
        winners: Vec::new(),
    });
    static TIMER_ID: RefCell<Option<TimerId>> = RefCell::new(None);
}

#[derive(CandidType, Deserialize, Debug)]
enum GameError {
    GameInProgress,
    GameNotInProgress,
    PlayerAlreadyHasCard,
    PlayerNotFound,
    NotEnoughPlayers,
}

#[update]
async fn generate_card() -> (Option<Card>, Option<GameError>) {
    let caller = ic_cdk::api::caller();
    let card = create_random_card(caller).await;
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        // Check if game is already in progress
        if state.is_active {
            return (None, Some(GameError::GameInProgress));
        }

        // Check if player already has a card
        if state.cards.contains_key(&caller) {
            return (None, Some(GameError::PlayerAlreadyHasCard));
        }

        state.cards.insert(caller, card.clone());

        // Auto-start game if we've reached required players
        if state.cards.len() == REQUIRED_PLAYERS {
            start_game_internal(&mut state);
        }

        (Some(card), None)
    })
}

#[update]
async fn reset_card() ->  (Option<Card>, Option<GameError>) {
    let caller = ic_cdk::api::caller();
    let new_card = create_random_card(caller).await;
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        
        // Check if game is in progress
        if state.is_active {
            return (None,Some(GameError::GameInProgress));
        }

        state.cards.insert(caller, new_card.clone());
        (Some(new_card), None)
    })
}


async fn create_random_card(owner: Principal) -> Card {
    let mut used_numbers = HashSet::new();
    let mut numbers = vec![vec![0; CARD_SIZE]; CARD_SIZE];

    // Generate 5x5 unique random numbers between 1 and 99
    let mut matrix_index = 0;
    let rand_numbers = raw_rand().await.unwrap();
    // Convert byte32 to array of numbers
    let random_numbers: Vec<u32> = rand_numbers.0.iter()
        .map(|&x| x as u32)
        .collect();

    // Create vector from 1 to 99
    let mut sequence: Vec<u32> = (1..=99).collect();

    let mut current_byte_index = 0;
    for i in (1..random_numbers.len()).rev() {
        let j = (random_numbers[current_byte_index] as usize) % (i + 1);
        sequence.swap(i, j);
        current_byte_index = (current_byte_index + 1) % 32;
    }
    // Calculate middle position
    let middle_row = CARD_SIZE / 2;
    let middle_col = CARD_SIZE / 2;

    for num in sequence {
        if used_numbers.insert(num) {  // returns true if number wasn't in set
            let i = matrix_index / CARD_SIZE;
            let j = matrix_index % CARD_SIZE;
            // Skip middle position
            if i == middle_row && j == middle_col {
                matrix_index += 1;
                continue;
            }
            numbers[i][j] = num;
            matrix_index += 1;

            if matrix_index >= CARD_SIZE * CARD_SIZE {
                break;
            }
        }
    }

    Card { numbers, owner }
}

fn start_game_internal(state: &mut GameState) {
    if state.cards.len() < REQUIRED_PLAYERS {
        return;
    }
    ic_cdk::api::print("Game started!");

    state.is_active = true;
    state.called_numbers.clear();
    state.called_numbers.insert(0);
    state.winners.clear();

    // Set up timer to generate numbers every 15 seconds
    let timer_id = set_timer_interval(Duration::from_secs(RANDOM_NUMBER_TIME_SECS), || {
        ic_cdk::spawn(async {
            generate_next_number().await;
        });
    });
    
    TIMER_ID.with(|id| *id.borrow_mut() = Some(timer_id));

}

async fn generate_next_number() {
    // Early return if game not active
    let is_active = STATE.with(|state| state.borrow().is_active);
    if !is_active {
        return;
    }
    ic_cdk::api::print("Random the next number!");

    // Get remaining numbers that haven't been called
    let called_numbers = STATE.with(|state| state.borrow().called_numbers.clone());
    let mut sequence: Vec<u32> = (1..=99).collect();
    sequence.retain(|num| !called_numbers.contains(num));

    if sequence.is_empty() {
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.is_active = false;
        });
        return;
    }

    // Shuffle sequence using random bytes
    let rand_numbers = raw_rand().await.unwrap();
    let random_numbers: Vec<u32> = rand_numbers.0.iter()
        .map(|&x| x as u32)
        .collect();

    let mut current_byte_index = 0;
    for i in (1..random_numbers.len()).rev() {
        let j = (random_numbers[current_byte_index] as usize) % (i + 1);
        sequence.swap(i, j);
        current_byte_index = (current_byte_index + 1) % 32;
    }

    // Take first number from shuffled sequence
    let new_number = sequence[0];

    // Update game state
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.called_numbers.insert(new_number);
        // check_winners(&mut state);
    });

}

#[update]
fn challenge() -> bool {
    let owner = ic_cdk::api::caller();
    STATE.with(|state| {
    let read_state = state.borrow().clone();
    let card = read_state.cards.get(&owner).unwrap();
    if is_winner(&card.numbers, &read_state.called_numbers){
        let mut global_state = state.borrow_mut();
        global_state.winners.push(owner);
        global_state.is_active = false;
        return true;
    }
    false
})
}

fn is_winner(card_numbers: &Vec<Vec<u32>>, called_numbers: &HashSet<u32>) -> bool {
    // Check rows
    for row in card_numbers {
        if row.iter().all(|num| called_numbers.contains(num)) {
            return true;
        }
    }

    // Check columns
    for col in 0..CARD_SIZE {
        if (0..CARD_SIZE).all(|row| called_numbers.contains(&card_numbers[row][col])) {
            return true;
        }
    }

    // Check diagonals
    if (0..CARD_SIZE).all(|i| called_numbers.contains(&card_numbers[i][i])) {
        return true;
    }
    if (0..CARD_SIZE).all(|i| called_numbers.contains(&card_numbers[i][4 - i])) {
        return true;
    }

    false
}

#[query]
fn get_game_state() -> GameState {
    STATE.with(|state| state.borrow().clone())
}

#[query]
fn get_player_count() -> usize {
    STATE.with(|state| state.borrow().cards.len())
}

#[query]
fn get_card() -> Option<Card> {
    let owner = ic_cdk::api::caller();
    STATE.with(|state| state.borrow().cards.get(&owner).cloned())
}

#[query]
fn get_remaining_slots() -> usize {
    STATE.with(|state| REQUIRED_PLAYERS - state.borrow().cards.len())
}