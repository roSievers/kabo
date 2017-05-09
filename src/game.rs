// This module contains the game state and messages.

use rand::{thread_rng, Rng};
use std::mem::swap;

// Types for everything that behaves like an object

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card(u8);

#[derive(Debug, Clone)]
pub struct Player {
    name: String,
    cards: Vec<Card>,
}

// Before the game starts, each player is allowed to peek twice.
// Stores how many peeks are left for each player.
#[derive(Debug, Clone)]
pub struct PreGame {
    deck: Vec<Card>,
    discard_pile: Vec<Card>,
    players: Vec<(Player, u8)>,
    total_peeks_left: u8,
}

#[derive(Debug, Clone)]
pub struct Game {
    deck: Vec<Card>,
    discard_pile: Vec<Card>,
    players: Vec<Player>,
    current_player: u8,
    kabo: Option<u8>,
    hand_card: Option<Card>,
}

// This should be an associated constant, once that feature stabilizes.
const DECK_SIZE: usize = 52;

impl Card {
    fn new(number: u8) -> Self {
        // Ideally this would be done with a dependent type.
        assert!(number <= 13, "The number {} is no valid card!", number);

        Card(number)
    }
    fn full_deck() -> Vec<Self> {
        let mut cards = Vec::with_capacity(DECK_SIZE);
        cards.push(Card::new(0));
        cards.push(Card::new(0));
        for i in 1..13 {
            for _ in 0..4 {
                cards.push(Card::new(i));
            }
        }
        cards.push(Card::new(13));
        cards.push(Card::new(13));

        thread_rng().shuffle(&mut cards);

        assert!(cards.len() == DECK_SIZE);
        cards
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreGameError {
    NoPeeksLeft,
    InvalidIndex,
}

impl PreGame {
    pub fn new(names: Vec<&str>, cards_per_player: u8) -> Self {
        let mut deck = Card::full_deck();
        let first_card = deck.pop().unwrap();
        let discard_pile = vec![first_card];

        let mut players = Vec::with_capacity(names.len());
        for name_borrow in names {
            let name: String = name_borrow.to_owned();
            let drain_index = deck.len() - cards_per_player as usize;
            let cards: Vec<Card> = deck.drain(drain_index..).collect();
            assert!(cards.len() == cards_per_player as usize);

            players.push((Player::new(name, cards), 2));
        }

        PreGame {
            deck,
            discard_pile,
            total_peeks_left: players.len() as u8 * 2,
            players,
        }
    }
    // Reveals to the player what card is hidden at a given location.
    // If they have no peeks left, it returns an error.
    // If they ask about an invalid index, it returns an error.
    // If an invalid player_index is supplied, it PANICS.
    // (because this value is not supplied by the client)
    pub fn peek(&mut self, player_index: u8, card_index: u8) -> Result<Card, PreGameError> {
        let ref mut player_tuple = self.players
            .get_mut(player_index as usize)
            .expect("Invalid player index.");
        let ref player = player_tuple.0;
        let ref mut peeks_left = player_tuple.1;

        if *peeks_left > 0 {
            *peeks_left -= 1;
            self.total_peeks_left -= 1;
            if let Some(card) = player.cards.get(card_index as usize) {
                Ok(card.clone())
            } else {
                Err(PreGameError::InvalidIndex)
            }
        } else {
            Err(PreGameError::NoPeeksLeft)
        }
    }
    pub fn to_game(mut self) -> Game {
        assert!(self.total_card_amout() == DECK_SIZE);
        assert!(self.total_peeks_left == 0);
        for ref player in &self.players {
            // Double check that there aren't any peeks left here either.
            assert!(player.1 == 0);
        }

        let players = self.players.drain(..).map(|x| x.0).collect();

        Game {
            deck: self.deck,
            discard_pile: self.discard_pile,
            players: players,
            current_player: 0,
            kabo: None,
            hand_card: None,
        }
    }
    // A test to assert that the number of cards doesn't change unexpectedly.
    fn total_card_amout(&self) -> usize {
        let mut total = self.deck.len() + self.discard_pile.len();
        for ref player in &self.players {
            total += player.0.cards.len()
        }

        total
    }
}

impl Player {
    fn new(name: String, cards: Vec<Card>) -> Self {
        Player { name, cards }
    }
}

macro_rules! ensure {
    ($condition:expr, $error:expr) => {{
        if !$condition {
            return Err($error);
        }
    }};
}

type Status = Result<Vec<GameEvent>, GameError>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GameEvent {
    DiscardShuffle,
    Discards { cards: Vec<Card> },
    Kabo { player_index: u8 },
    EndTurn { next_player: u8 },
    Seen {
        player_index: u8,
        card_index: u8,
        card: Card,
    },
    GameOver,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GameError {
    WrongPhase,
    AlreadyKabo { player_index: u8 },
    WrongCard,
    InvalidIndex,
}


#[allow(dead_code)]
impl Game {
    // Draw a card from the deck. If there is none, shuffle the discard pile as new deck.
    // The functions return Err(()) if the move is illegal, but have no way to check if they
    // were send by the corret player. This has to be verified externally.
    pub fn deck_draw(&mut self) -> Status {
        ensure!(self.hand_card.is_none(), GameError::WrongPhase);

        if let Some(card) = self.deck.pop() {
            self.hand_card = Some(card);
            Ok(vec![])
        } else {
            // The deck is empty, shuffle the discard pile
            assert!(self.discard_pile.len() >= 4);
            swap(&mut self.deck, &mut self.discard_pile);
            thread_rng().shuffle(&mut self.deck);
            // Yes, there is no remaining top card, but after drawing the player
            // is certain to discard a card.

            self.hand_card = self.deck.pop();
            Ok(vec![GameEvent::DiscardShuffle])
        }
    }
    pub fn discard_draw(&mut self) -> Status {
        ensure!(self.hand_card.is_none(), GameError::WrongPhase);

        // This unwraps an Option<Card> and then it immediately rewraps it.
        // I do this to assert that there really is a card. If for some reason
        // the discard pile doesn't contain any cards, then this will panic.
        // It also is more clear that we are dealing with an Option value.
        self.hand_card = Some(self.discard_pile.pop().unwrap());
        Ok(vec![])
    }
    #[allow(dead_code)]
    pub fn announce_kabo(&mut self) -> Status {
        ensure!(self.hand_card.is_none(), GameError::WrongPhase);
        if let Some(player_index) = self.kabo {
            return Err(GameError::AlreadyKabo { player_index });
        };

        self.kabo = Some(self.current_player);
        let kabo_event = GameEvent::Kabo { player_index: self.current_player };
        Ok(vec![kabo_event, self.end_turn()])
    }
    pub fn discard(&mut self) -> Status {
        ensure!(self.hand_card.is_some(), GameError::WrongPhase);

        Ok(self.discard_and_end())
    }
    #[allow(dead_code)]
    pub fn replace(&mut self, player_index: u8, card_index: u8) -> Status {
        ensure!(self.hand_card.is_some(), GameError::WrongPhase);

        {
            // Enclose all references to self in an environment to satisfy
            // the borrow checker.
            let hand_card = self.hand_card.as_mut().unwrap();

            if let Some(face_down_card) =
                self.players[player_index as usize]
                    .cards
                    .get_mut(card_index as usize) {

                swap(hand_card, face_down_card);
            } else {
                return Err(GameError::InvalidIndex);
            };
        }

        Ok(self.discard_and_end())

    }
    pub fn multi_replace(&mut self, player_index: u8, card_indices: Vec<u8>) -> Status {
        ensure!(card_indices.len() >= 2, GameError::InvalidIndex);
        ensure!(self.hand_card.is_some(), GameError::WrongPhase);

        unimplemented!()
    }
    pub fn peek(&mut self, player_index: u8, card_index: u8) -> Status {
        ensure!(self.hand_card.is_some(), GameError::WrongPhase);
        let card = self.hand_card.unwrap();
        ensure!(card.0 == 7 || card.0 == 8, GameError::WrongCard);

        unimplemented!()
    }
    // End turn can't be called manually. Any code that calls it has already
    // checked if the request is good so this can't return an error.
    fn end_turn(&mut self) -> GameEvent {
        // End turn can't be called from the outside so this indicates a bug and panics.
        assert!(self.hand_card.is_none(),
                "Inconsistent state while ending Turn.");

        self.current_player += 1;
        if self.current_player as usize == self.players.len() {
            self.current_player = 0;
        }
        if let Some(kabo_index) = self.kabo {
            if kabo_index == self.current_player {
                return GameEvent::GameOver;
            }
        }
        GameEvent::EndTurn { next_player: self.current_player }
    }
    // Panics, if there is no hand card.
    fn discard_and_end(&mut self) -> Vec<GameEvent> {
        let card = self.hand_card.unwrap();
        self.discard_pile.push(card);
        self.hand_card = None;

        vec![GameEvent::Discards { cards: vec![card] }, self.end_turn()]
    }
}
