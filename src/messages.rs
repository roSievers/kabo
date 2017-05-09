use game::{GameEvent, Card};

pub enum Action {
    // Before you hold a card
    DeckDraw,
    DiscardDraw,
    Kabo,
    // When you hold a card
    Replace { card_index: u8 },
    MultiReplace {
        card_type: Card,
        card_indices: Vec<u8>,
    },
    Peek { card_index: u8 },
    Spy {
        other_player_index: u8,
        card_index: u8,
    },
    Swap {
        my_card_index: u8,
        other_player_index: u8,
        other_card_index: u8,
    },
    Discard,
}

#[derive(Debug)]
pub enum Player {
    // At any time:
    AskState { player_index: u8 },
    Play { action: Action },
}

#[derive(Debug)]
pub enum Server {
    // Public information
    StartTurn { player_index: u8 },
    ActionSuccess {
        player_index: u8,
        action: Action,
        discards: Vec<Card>,
    },
    MultiReplaceFailure {
        player_index: u8,
        card_type_claimed: Card,
        cards_seen: Vec<(u8, Card)>,
    },
    // Private information
    CardDrawn { card: Card },
    CardSeen {
        player_index: u8,
        card_index: u8,
        card: Card,
    },
    Error { error: GameError },
}
