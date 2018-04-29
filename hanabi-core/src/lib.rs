//! hanabi-core
//! rules and data-structures for hanabi

#[macro_use]
extern crate enum_iterator_derive;
extern crate error_chain_mini;
#[macro_use]
extern crate error_chain_mini_derive;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate uuid;

use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
/// colors of hanabi cards
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, EnumIterator)]
pub enum Color {
    White,
    Red,
    Blue,
    Yellow,
    Green,
    Multi,
}

impl Color {
    pub fn to_usize(&self) -> usize {
        match *self {
            White => 0,
            Red => 1,
            Blue => 2,
            Yellow => 3,
            Green => 4,
            Multi => 5,
        }
    }
}

/// unique identifier of cards
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CardId(Uuid);

/// hanabi card
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub number: usize,
    pub color: Color,
    pub card_id: CardId,
}

impl Card {
    fn new(n: usize, c: Color) -> Self {
        let uuid = Uuid::new_v4();
        Card {
            number: n,
            color: c,
            card_id: CardId(uuid),
        }
    }
}

/// token
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Token {
    Blue,
    Red,
}

/// multiple tokens
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NumberdToken {
    pub num: usize,
    pub kind: Token,
}

/// Player information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub player_id: usize,
    pub hands: Vec<Card>,
}

impl Player {
    fn new(id: usize, hands: Vec<Card>) -> Self {
        Player {
            player_id: id,
            hands,
        }
    }
}

/// Player Action
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Tell(CardInfo),
    Discard(CardId),
    Play(CardId),
}

/// what to tell
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CardInfo {
    Color { color: Color, cards: Vec<CardId> },
    Number { number: usize, cards: Vec<CardId> },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Field {
    inner: Vec<Vec<Card>>,
}

impl Field {
    pub fn add(&mut self, card: Card) -> bool {
        let id = card.color.to_usize();
        let last_number = match self.inner[id].last() {
            Some(card) => card.number,
            None => 0,
        };
        if last_number + 1 == card.number {
            self.inner[id].push(card);
            true
        } else {
            false
        }
    }
}

/// game information(runtime)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    /// player informations
    pub players: Vec<Player>,
    /// stack of cards
    pub stack: Vec<Card>,
    /// discarded cards
    pub discards: Vec<Card>,
    /// hanabi field
    pub field: Field,
    player_num: usize,
    is_multi: bool,
    is_grand_finale: bool,
}

impl Game {
    fn process_action(&mut self, player: usize, act: Action) {}
}

/// game config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    player_num: usize,
    is_multi: bool,
    is_grand_finale: bool,
}

impl Config {
    pub fn new(n: usize) -> Option<Config> {
        if n < 2 || n > 5 {
            return None;
        }
        Some(Config {
            player_num: n,
            is_multi: false,
            is_grand_finale: false,
        })
    }
    pub fn multi(&mut self, f: bool) -> &mut Self {
        self.is_multi = f;
        self
    }
    pub fn grand_finale(&mut self, f: bool) -> &mut Self {
        self.is_grand_finale = f;
        self
    }
    pub fn build(self) -> Game {
        let n = self.player_num;
        let is_multi = self.is_multi;
        let (player_cards, stack) = prepare_cards(n, is_multi);
        let players: Vec<_> = player_cards
            .into_iter()
            .enumerate()
            .map(|(i, hands)| Player::new(i, hands))
            .collect();
        Game {
            players,
            stack,
            discards: Vec::new(),
            field: Field::default(),
            player_num: n,
            is_multi,
            is_grand_finale: self.is_grand_finale,
        }
    }
}

fn prepare_cards(n: usize, is_multi: bool) -> (Vec<Vec<Card>>, Vec<Card>) {
    let mut stack_cards: Vec<_> = {
        let card_kinds = if is_multi { 5 } else { 6 };
        Color::iter_variants()
            .take(card_kinds)
            .flat_map(|var| (1..6).map(|num| Card::new(num, var)).collect::<Vec<_>>())
            .collect()
    };
    let mut rng = rand::thread_rng();
    rng.shuffle(&mut stack_cards);
    let card_num = if n <= 3 { 5 } else { 4 };
    let mut hands = vec![vec![]; n];
    for _ in 0..card_num {
        for i in 0..n {
            let card = stack_cards.pop().unwrap();
            hands[i].push(card)
        }
    }
    (hands, stack_cards)
}

#[derive(Clone, Debug, ErrorKind)]
pub enum CoreError {
    #[msg(short = "invalid card id", detailed = "{:?}", _0)]
    InvalidCard(Uuid),
    #[msg(short = "invalid player id", detailed = "{:?}", _0)]
    InvalidPlayer(usize),
}
