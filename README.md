# ICP Bingo Game for 2024 Chain Fusion Hacker House Devcon Bangkok

A decentralized Bingo game implementation running on the Internet Computer Protocol (ICP).

## Overview

This project implements a multiplayer Bingo game where:

- Each player gets a 5x5 card with random numbers
- The middle square is free
- Numbers are called automatically every 15 seconds
- First player who challenges and completed a row, column, or diagonal wins

## Features

- Automatic game start when required number of players join
- Random number generation using IC's randomness
- Automated number calling via IT's timer
- Winner verification for rows, columns, and diagonals
- Real-time game state updates

## Technical Details

- Built on Internet Computer Protocol (ICP)
- Written in Rust using Candid interface description language
- Uses thread-local storage for game state management

## Game Rules

1. Players join by generating their cards
2. Game starts automatically when required number of players join
3. A number is generated every 15 seconds
4. Players can challenge for a win when they think they have won
5. Game ends when a valid win is verified or runs out of the numbers

## API Functions

### Player Actions

- `generate_card()`: Get a new bingo card
- `reset_card()`: Reset your current card (only before game starts)
- `challenge()`: Submit a win challenge

### Query Functions

- `get_game_state()`: Get current game state
- `get_player_count()`: Get number of players
- `get_card()`: Get your current card
- `get_remaining_slots()`: Check available player slots

## Setup

1. Install the DFINITY Canister SDK
2. Clone this repository
3. Deploy using:
   `dfx deploy`

## Usage Example

```bash
# Generate a new card
dfx canister call bingo generate_card

# Check game state
dfx canister call bingo get_game_state

# Challenge for win
dfx canister call bingo challenge
```
