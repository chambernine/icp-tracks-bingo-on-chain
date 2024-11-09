use candid::{CandidType, Principal};
use ic_cdk::{query, update};
use ic_cdk_timers::{set_timer_interval, TimerId};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use serde::Deserialize;
use ic_cdk::api::management_canister::main::raw_rand;
// use serde_json::{self};

const REQUIRED_PLAYERS: usize = 10;
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
async fn generate_card(caller: Principal) -> (Option<Card>, Option<GameError>) {
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
async fn reset_card(caller: Principal) ->  (Option<Card>, Option<GameError>) {
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
    let min = 1;
    let max = 99;
    let mut used_numbers = HashSet::new();
    let mut numbers = vec![vec![0; CARD_SIZE]; CARD_SIZE];

    // Generate 5x5 unique random numbers between 1 and 75
    for i in 0..CARD_SIZE {
        for j in 0..CARD_SIZE {
            loop {
                let bytes = raw_rand().await.unwrap().0;
                let num_u32 = u32::from_le_bytes(bytes[0..8].try_into().unwrap());
                let num = (num_u32 % (max - min + 1) as u32) + min as u32;
                if used_numbers.insert(num) {
                    numbers[i][j] = num;
                    break;
                }
            }
        }
    }

    Card { numbers, owner }
}

fn start_game_internal(state: &mut GameState) {
    if state.cards.len() < REQUIRED_PLAYERS {
        return;
    }

    state.is_active = true;
    state.called_numbers.clear();
    state.winners.clear();

    // Set up timer to generate numbers every 15 seconds
    let timer_id = set_timer_interval(Duration::from_secs(15), || {
        ic_cdk::spawn(async {
            generate_next_number().await;
        });
    });
    
    TIMER_ID.with(|id| *id.borrow_mut() = Some(timer_id));

}

async fn generate_next_number() {
    let min = 1;
    let max = 99;
    
    // Early return if game not active
    let is_active = STATE.with(|state| state.borrow().is_active);
    if !is_active {
        return;
    }

    // Generate initial random number
    let bytes = raw_rand().await.unwrap().0;
    let mut new_number = {
        let num_u32 = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        (num_u32 % (max - min + 1) as u32) + min as u32
    };

    // Keep generating new numbers until we find an unused one
    loop {
        let should_continue = STATE.with(|state| {
            let mut state = state.borrow_mut();
            
            if state.called_numbers.contains(&new_number) {
                if state.called_numbers.len() >= 99 {
                    state.is_active = false;
                    return false;
                }
                return true;
            }
            
            state.called_numbers.insert(new_number);
            check_winners(&mut state);
            false
        });

        if !should_continue {
            break;
        }

        // Generate next random number
        let bytes = raw_rand().await.unwrap().0;
        let num_u32 = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        new_number = (num_u32 % (max - min + 1) as u32) + min as u32;
    }
}

fn check_winners(state: &mut GameState) {
    for card in state.cards.values() {
        if is_winner(&card.numbers, &state.called_numbers) {
            state.winners.push(card.owner);
            state.is_active = false;
            
            // Clear timer when game ends
            if let Some(timer_id) = TIMER_ID.with_borrow(|id| id.clone()) {
                ic_cdk_timers::clear_timer(timer_id);
            }
            break;
        }
    }
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
fn get_card(owner: Principal) -> Option<Card> {
    STATE.with(|state| state.borrow().cards.get(&owner).cloned())
}

#[query]
fn get_remaining_slots() -> usize {
    STATE.with(|state| REQUIRED_PLAYERS - state.borrow().cards.len())
}