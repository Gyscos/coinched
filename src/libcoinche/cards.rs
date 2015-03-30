extern crate rand;

use self::rand::{thread_rng,Rng};
use std::num::wrapping::Wrapping;

#[derive(PartialEq,Copy)]
pub struct Suit(u32);
pub const HEART: Suit = Suit(1 << 0);
pub const SPADE: Suit = Suit(1 << 8);
pub const DIAMOND: Suit = Suit(1 << 16);
pub const CLUB: Suit = Suit(1 << 24);

pub fn get_suit(n: u32) -> Suit {
    Suit(1 << 8*n)
}

impl Suit {
    pub fn to_string(self) -> String {
        match self {
            HEART => "♥",
            SPADE => "♠",
            DIAMOND => "♦",
            CLUB => "♣",
            _ => "?",
        }.to_string()
    }
}


#[derive(PartialEq,Copy)]
pub struct Rank(u32);
pub const RANK_7: Rank = Rank(1 << 0);
pub const RANK_8: Rank = Rank(1 << 1);
pub const RANK_9: Rank = Rank(1 << 2);
pub const RANK_J: Rank = Rank(1 << 3);
pub const RANK_Q: Rank = Rank(1 << 4);
pub const RANK_K: Rank = Rank(1 << 5);
pub const RANK_X: Rank = Rank(1 << 6);
pub const RANK_A: Rank = Rank(1 << 7);
pub const RANK_MASK: Rank = Rank(255);

pub fn get_rank(n: u32) -> Rank {
    Rank(1 << n)
}

impl Rank {
    pub fn to_string(self) -> String {
        match self {
            RANK_7 => "7",
            RANK_8 => "8",
            RANK_9 => "9",
            RANK_J => "J",
            RANK_Q => "Q",
            RANK_K => "K",
            RANK_X => "X",
            RANK_A => "A",
            _ => "?",
        }.to_string()
    }
}

#[derive(PartialEq,Copy)]
pub struct Card(u32);

impl Card {
    pub fn id(self) -> u32 {
        let mut i = 0;
        let Card(mut v) = self;
        while v != 0 {
            i+=1;
            v = v>>1;
        }

        i-1
    }

    pub fn rank(self) -> Rank {
        let Card(mut v) = self;
        let mut r: u32 = 0;
        let Rank(mask) = RANK_MASK;

        r |= mask & v;
        v = v >> 8;
        r |= mask & v;
        v = v >> 8;
        r |= mask & v;
        v = v >> 8;
        r |= v;

        Rank(r)
    }

    pub fn suit(self) -> Suit {
        let Card(v) = self;
        let Rank(r) = self.rank();
        Suit(v / r)
    }

    pub fn to_string(self) -> String {
        let r = self.rank();
        let s = self.suit();
        r.to_string() + &s.to_string()
    }
}

pub fn make_card(suit: Suit, rank: Rank) -> Card {
    let Suit(s) = suit;
    let Rank(r) = rank;

    Card(s * r)
}

pub fn get_card(id: u32) -> Card {
    Card(1 << id)
}

#[test]
fn card_test() {
    for i in 0..32 {
        let card = get_card(i);
        assert!(i == card.id());
    }

    for s in 0..4 {
        let suit = get_suit(s);
        for r in 0..8 {
            let rank = get_rank(r);
            let card = make_card(suit, rank);
            assert!(card.rank() == rank);
            assert!(card.suit() == suit);
        }
    }
}

#[derive(PartialEq,Copy)]
pub struct Hand(u32);

pub fn new_hand() -> Hand {
    Hand(0)
}

impl Hand {
    pub fn add(&mut self, card: Card) {
        self.0 |= card.0;
    }

    pub fn remove(&mut self, card: Card) {
        self.0 &= !card.0;
    }

    pub fn has(self, card: Card) -> bool {
        (self.0 & card.0) != 0
    }

    pub fn has_any(self, suit: Suit) -> bool {
        (self.0 & (RANK_MASK.0 * suit.0)) != 0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn get_card(self) -> Card {
        if self.is_empty() {
            return Card(0);
        }

        let Hand(h) = self;
        // Finds the rightmost bit, shifted to the left by 1.
        // let n = 1 << (h.trailing_zeroes());
        let n = Wrapping(h ^ (h - 1)) + Wrapping(1);
        if n.0 == 0 {
            // We got an overflow. This means the desired bit it the leftmost one.
            get_card(31)
        } else {
            // We just need to shift it back.
            Card(n.0 >> 1)
        }
    }

    pub fn list(self) -> Vec<Card> {
        let mut cards = Vec::new();
        let mut h = self;

        while !h.is_empty() {
            let c = h.get_card();
            h.remove(c);
            cards.push(c);
        }

        cards
    }

    pub fn size(self) -> usize {
        self.list().len()
    }

    pub fn to_string(self) -> String {
        let mut s = "[".to_string();

        for c in self.list().iter() {
            s = s + &c.to_string();
            s = s +",";
        }

        s + "]"
    }
}

#[test]
fn hand_test() {
    let mut hand = new_hand();

    let cards: Vec<Card> = vec![
        make_card(HEART, RANK_7),
        make_card(HEART, RANK_8),
        make_card(SPADE, RANK_9),
        make_card(SPADE, RANK_J),
        make_card(CLUB, RANK_Q),
        make_card(CLUB, RANK_K),
        make_card(DIAMOND, RANK_X),
        make_card(DIAMOND, RANK_A),
    ];

    assert!(hand.is_empty());

    for card in cards.iter() {
        assert!(!hand.has(*card));
        hand.add(*card);
        assert!(hand.has(*card));
    }

    assert!(hand.size() == cards.len());

    for card in cards.iter() {
        assert!(hand.has(*card));
        hand.remove(*card);
        assert!(!hand.has(*card));
    }
}

pub struct Deck{
    cards: Vec<Card>,
}

// Returns a full, sorted deck of 32 cards.
pub fn new_deck() -> Deck {
    let mut d = Deck{cards:Vec::with_capacity(32)};

    for i in 0..32 {
        d.cards.push(get_card(i));
    }

    d
}

impl Deck {
    pub fn shuffle(&mut self) {
        thread_rng().shuffle(&mut self.cards[..]);
    }

    pub fn draw(&mut self) -> Card {
        self.cards.pop().expect("deck is empty")
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn to_string(&self) -> String {
        let mut s = "[".to_string();

        for c in self.cards.iter() {
            s = s + &c.to_string();
            s = s +",";
        }

        s + "]"
    }
}

#[test]
fn test_deck() {
    let mut deck = new_deck();
    deck.shuffle();

    assert!(deck.len() == 32);

    let mut count = [0; 32];
    while !deck.is_empty() {
        let card = deck.draw();
        count[card.id() as usize] += 1;
    }

    for c in count.iter() {
        assert!(*c == 1);
    }
}

pub fn deal_each(d: &mut Deck, hands: &mut [Hand; 4], n: usize) {
    if d.len() < 4*n {
        panic!("Deck has too few cards!");
    }

    for hand in hands.iter_mut() {
        for _ in 0..n {
            hand.add(d.draw());
        }
    }
}

pub fn deal_hands() -> [Hand; 4] {
    let mut hands = [new_hand(); 4];

    let mut d = new_deck();
    d.shuffle();

    deal_each(&mut d, &mut hands, 3);
    deal_each(&mut d, &mut hands, 2);
    deal_each(&mut d, &mut hands, 3);

    hands
}

#[test]
fn test_deals() {
    let hands = deal_hands();

    let mut count = [0; 32];
    for hand in hands.iter() {
        assert!(hand.size() == 8);
        for card in hand.list().iter() {
            count[card.id() as usize] += 1;
        }
    }

    for c in count.iter() {
        assert!(*c == 1);
    }
}