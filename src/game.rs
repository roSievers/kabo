// This module contains the game state and messages.

use rand::{thread_rng, Rng};

// Types for everything that behaves like an object

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card(u8);

#[derive(Debug)]
pub struct Deck(Vec<Card>);

#[derive(Debug)]
pub struct Player {
    name: String,
    cards: Vec<Card>,
    state: PlayerState,
}

#[derive(Debug)]
pub struct Game {
    pub deck: Deck,
    pub discard_pile: Vec<Card>,
    pub players: Vec<Player>,
    pub current_player: usize,
}

// Each player has a finite state machine attached to keep track of what he is allowed to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    // In each preparation phase you may Peek once.
    FirstPreparation,
    SecondPreparation,
    NotYourTurn,
    YourTurn,
    UseCard(Card),
    Kabo,
}

impl Card {
    fn new(number: u8) -> Card {
        // Ideally this would be done with a dependent type.
        assert!(number <= 13, "The number {} is no valid card!", number);

        Card(number)
    }
}

impl Deck {
    fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        cards.push(Card::new(0));
        cards.push(Card::new(0));
        for i in 1..13 {
            for _ in 0..4 {
                cards.push(Card::new(i));
            }
        }
        cards.push(Card::new(13));
        cards.push(Card::new(13));

        assert!(cards.len() == 52);

        Deck(cards).shuffle()
    }
    fn drain_discard_pile(cards: &mut Vec<Card>) -> Self {
        Deck(cards.drain(..).collect()).shuffle()
    }
    fn shuffle(mut self) -> Self {
        thread_rng().shuffle(&mut self.0);
        self
    }
    fn pop(&mut self) -> Option<Card> {
        self.0.pop()
    }
    // Panics if there aren't enought cards.
    fn deal_cards(&mut self, players: &mut Vec<Player>, cards_per_player: u8) {
        assert!(self.0.len() >= players.len() * cards_per_player as usize);
        for _ in 0..cards_per_player {
            for player in players.iter_mut() {
                let card = self.pop().unwrap();
                player.cards.push(card);
            }
        }
    }
}

impl Player {
    fn new(name: String) -> Self {
        Player {
            name: name,
            cards: Vec::new(),
            state: PlayerState::FirstPreparation,
        }
    }
}

impl Game {
    pub fn new(names: Vec<&str>) -> Self {
        // There are only 4 * 13 cards in the game. If everyone gets four and at
        // least two remain in the middle, then up to 12 players are possible.
        assert!(names.len() <= 12,
                "To many players. There are {}, but only 12 are allowed",
                names.len());

        let mut players = names
            .iter()
            .map(|name| Player::new((*name).to_owned()))
            .collect();
        let mut deck = Deck::new();

        deck.deal_cards(&mut players, 4);

        let first_card = deck.pop().unwrap();
        let discard_pile = vec![first_card];

        Game {
            deck: deck,
            discard_pile: discard_pile,
            players: players,
            current_player: 0,
        }
    }
    // This huge function implements the game mechanics. (Might refactor into smaller functions.)
    // It takes the player Message modifies the state
    // Returns a reply message and might return a message for all other players.
    pub fn update(&mut self,
                  sender_index: u8,
                  message: PlayerMessage)
                  -> (ServerMessage, Option<BroadcastMessage>) {

        // Use this macro to ensure that the player is even allowed to send the
        // message they just submitted.
        macro_rules! state_guard(
            ($state:ident) => {{
                let ref mut player = self.players[sender_index as usize];
                if player.state != PlayerState::$state {
                    return (ServerMessage::IllegalMessage {
                                state: player.state,
                                message: message,
                            },
                            None);
                }
            }}
        );

        match message {
            PlayerMessage::AskState { player_index } => {
                if let Some(player) = self.players.get(player_index as usize) {
                    (ServerMessage::StateIs {
                         player_index: player_index,
                         state: player.state,
                     },
                     None)
                } else {
                    (ServerMessage::IllegalIndex {
                         index: player_index,
                         bound: self.players.len() as u8,
                     },
                     None)
                }
            }
            PlayerMessage::DeckDraw => {
                state_guard!(YourTurn);

                let card = self.deck_draw();
                self.players[sender_index as usize].state = PlayerState::UseCard(card);

                (ServerMessage::DrawResult { card: card }, Some(BroadcastMessage::DeckDraw))
            }
            _ => {
                // TODO: All other messages :P
                unimplemented!();
            }
        }
    }
    // Draw a card from the deck. If there is none, shuffle the discard pile as new deck.
    fn deck_draw(&mut self) -> Card {
        if let Some(card) = self.deck.pop() {
            card
        } else {
            // The deck is empty, shuffle the discard pile
            assert!(self.discard_pile.len() >= 4);
            self.deck = Deck::drain_discard_pile(&mut self.discard_pile);
            // Yes, there is no remaining top card, but after drawing the player
            // is certain to discard a card.

            self.deck.pop().unwrap()
        }
    }
}

// Message types

#[derive(Debug)]
pub enum PlayerMessage {
    // At any time:
    AskState { player_index: u8 },
    // In PlayerState::YourTurn
    DeckDraw,
    DiscardDraw,
    Kabo,
    // In PlayerState::UseCard
    Replace { indices: Vec<u8> },
    Peek { index: u8 },
    Spy { player_index: u8, card_index: u8 },
    Swap {
        my_index: u8,
        player_index: u8,
        card_index: u8,
    },
    Discard,
}

// A broadcast message is always related to a player.
// It is send to all other players to inform them about the current state of the game.
#[derive(Debug)]
pub enum BroadcastMessage {
    StartTurn,
    DeckDraw,
    DiscardDraw,
    Kabo,
    // In PlayerState::UseCard
    Replace {
        indices: Vec<u8>,
        discard: Vec<Card>,
    },
    ReplaceFailiure {
        indices: Vec<u8>,
        cards: Vec<Card>,
        discard: Card,
    },
    Peek { index: u8, discard: Card },
    Spy {
        player_index: u8,
        card_index: u8,
        discard: Card,
    },
    Swap {
        my_index: u8,
        player_index: u8,
        card_index: u8,
        discard: Card,
    },
    Discard { discard: Card },
}

#[derive(Debug)]
// This server message is directed to a player and unless otherwise specified
// it is about their actions.
pub enum ServerMessage {
    StateIs {
        player_index: u8,
        state: PlayerState,
    },
    DrawResult { card: Card },
    KaboConfirmation,
    ReplaceConfirmation { indices: Vec<u8>, cards: Vec<Card> },
    ReplaceFailiure { indices: Vec<u8>, cards: Vec<Card> },
    PeekResult { index: u8, card: Card },
    SpyResult {
        player_index: u8,
        card_index: u8,
        card: Card,
    },
    SwapConfirmation {
        my_index: u8,
        player_index: u8,
        card_index: u8,
    },
    DiscardConfirmation,
    IllegalMessage {
        message: PlayerMessage,
        state: PlayerState,
    },
    IllegalIndex { index: u8, bound: u8 },
    OtherPlayer {
        player_index: u8,
        message: BroadcastMessage,
    },
    Timeout,
}
