// Thinking about the same thing with types.

extern crate rand;

use rand::{thread_rng, Rng};

// Types for everything that behaves like an object

#[derive(Debug, Clone, Copy)]
struct Card(u8);

#[derive(Debug)]
struct Deck(Vec<Card>);

#[derive(Debug)]
struct Player {
    name: String,
    cards: Vec<Card>,
    state: PlayerState,
}

#[derive(Debug)]
struct Game {
    deck: Deck,
    discard_pile: Vec<Card>,
    players: Vec<Player>,
    current_player: usize,
}

// Each player has a finite state machine attached to keep track of what he is allowed to do.
#[derive(Debug, Clone)]
enum PlayerState {
    // In each preparation phase you may Peek once.
    FirstPreparation,
    SecondPreparation,
    NotYourTurn,
    YourTurn,
    UseCard(Card),
    Kabo,
}

impl Card {
    pub fn new(number: u8) -> Card {
        // Ideally this would be done with a dependent type.
        assert!(number <= 13, "The number {} is no valid card!", number);

        Card(number)
    }
}

impl Deck {
    pub fn new() -> Self {
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
    pub fn shuffle(mut self) -> Self {
        thread_rng().shuffle(&mut self.0);
        self
    }
    pub fn pop(&mut self) -> Option<Card> {
        self.0.pop()
    }
    // Panics if there aren't enought cards.
    pub fn deal_cards(&mut self, players: &mut Vec<Player>, cards_per_player: u8) {
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
    pub fn new(name: String) -> Self {
        Player {
            name: name,
            cards: Vec::new(),
            state: PlayerState::FirstPreparation,
        }
    }
}

impl Game {
    pub fn new(names: Vec<&str>) -> Self {
        // There are only 4 * 13 cards in the game. If everyone gets one and at
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
}

// Message types

#[derive(Debug)]
enum PlayerMessage {
    // At any time:
    AskState,
    // In PlayerState::YourTurn
    DeckDraw,
    DiscardDraw,
    Kabo,
    // In PlayerState::UseCard
    Replace { index: u8 },
    Peek { index: u8 },
    Spy { player_index: u8, card_index: u8 },
    Swap {
        my_index: u8,
        player_index: u8,
        card_index: u8,
    },
    Discard,
}

#[derive(Debug)]
enum ServerMessage {
    StateIs { state: PlayerState },
    DrawResult { card: Card },
    KaboConfirmation,
    Replace { index: u8, card: Card },
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
}

fn main() {
    println!("Hello, world!");

    let mut game = Game::new(vec!["Judita", "Sara", "Rolf"]);

    println!("The game is: {:?}", game);
}
