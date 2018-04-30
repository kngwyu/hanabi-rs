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

mod error;

use error::{CoreError, Error, ErrorKind, ResultExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::collections::HashSet;
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
            Color::White => 0,
            Color::Red => 1,
            Color::Blue => 2,
            Color::Yellow => 3,
            Color::Green => 4,
            Color::Multi => 5,
        }
    }
}

/// number of hanabi cards
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, EnumIterator)]
pub enum Number {
    One,
    Two,
    Three,
    Four,
    Five,
}

impl Number {
    pub fn to_usize(&self) -> usize {
        match *self {
            Number::One => 1,
            Number::Two => 2,
            Number::Three => 3,
            Number::Four => 4,
            Number::Five => 5,
        }
    }
}

/// unique identifier of cards
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CardId(Uuid);

/// hanabi card
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub number: Number,
    pub color: Color,
    pub id: CardId,
}

impl Card {
    fn new(n: Number, c: Color) -> Self {
        let uuid = Uuid::new_v4();
        Card {
            number: n,
            color: c,
            id: CardId(uuid),
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

/// Player information in the game
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub player_id: usize,
    pub hands: Vec<Card>,
}

impl PlayerInfo {
    fn new(id: usize, hands: Vec<Card>) -> Self {
        PlayerInfo {
            player_id: id,
            hands,
        }
    }
    pub fn card_idx(&self, id: CardId) -> Option<usize> {
        self.hands
            .iter()
            .enumerate()
            .find(|(_, card)| card.id == id)
            .map(|(i, _)| i)
    }
    pub fn remove_card(&mut self, idx: usize) -> Card {
        self.hands.remove(idx)
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
pub struct CardInfo {
    kind: CardInfoKind,
    player: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CardInfoInner {
    kind: CardInfoKind,
    cards: HashSet<CardId>,
    player: usize,
}

/// kind of card information
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum CardInfoKind {
    Color(Color),
    Number(Number),
}

/// cards in field
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Field {
    inner: Vec<Vec<Card>>,
}

impl Field {
    fn new() -> Self {
        Field {
            inner: vec![vec![]; 6],
        }
    }
    pub fn add(&mut self, card: Card) -> bool {
        let id = card.color.to_usize();
        let last_number = match self.inner[id].last() {
            Some(card) => card.number.to_usize(),
            None => 0,
        };
        if last_number + 1 == card.number.to_usize() {
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
    pub players: Vec<PlayerInfo>,
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
    fn process_action(
        &mut self,
        player: usize,
        act: Action,
    ) -> Result<Option<CardInfoInner>, Error> {
        if !self.is_valid_player(player) {
            return Err(CoreError::InvalidPlayer(player).into_err());
        }
        macro_rules! get_card {
            ($id:ident) => {
                if let Some(idx) = self.players[player].card_idx($id) {
                    self.players[player].remove_card(idx)
                } else {
                    return Err(CoreError::InvalidCard($id).into_err());
                }
            };
        }
        let res = match act {
            Action::Discard(id) => {
                let card = get_card!(id);
                self.discards.push(card);
                None
            }
            Action::Play(id) => {
                let card = get_card!(id);
                if !self.field.add(card) {
                    return Err(CoreError::InvalidCard(id).into_err());
                }
                None
            }
            Action::Tell(ref info) => if let Some(cards) = self.construct_info(info) {
                if cards.is_empty() {
                    return Err(CoreError::IncorrectInfo(info.to_owned()).into_err());
                }
                let info_inner = CardInfoInner {
                    kind: info.kind,
                    player: info.player,
                    cards,
                };
                Some(info_inner)
            } else {
                return Err(CoreError::IncorrectInfo(info.to_owned()).into_err());
            },
        };
        Ok(res)
    }
    fn is_valid_player(&self, n: usize) -> bool {
        n < self.player_num
    }
    fn construct_info(&self, info: &CardInfo) -> Option<HashSet<CardId>> {
        if !self.is_valid_player(info.player) {
            return None;
        }
        let is_valid_card = |card: &&Card| match info.kind {
            CardInfoKind::Color(c) => card.color == c,
            CardInfoKind::Number(n) => card.number == n,
        };
        Some(
            self.players[info.player]
                .hands
                .iter()
                .filter(is_valid_card)
                .map(|card| card.id)
                .collect(),
        )
    }
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
            .map(|(i, hands)| PlayerInfo::new(i, hands))
            .collect();
        Game {
            players,
            stack,
            discards: Vec::new(),
            field: Field::new(),
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
            .flat_map(|var| {
                Number::iter_variants()
                    .map(|num| Card::new(num, var))
                    .collect::<Vec<_>>()
            })
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
