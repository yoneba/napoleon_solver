type Suit = u32;
type Rank = u64;
type Cards = u64;
type CardIndex = usize;
type PlayerIndex = usize;
type Declaration = usize;

const N_CARDS: CardIndex = 55;

const SPADE: Suit = 0;
const HEART: Suit = 16;
const DIAMOND: Suit = 32;
const CLUB: Suit = 48;

const TWO: Rank = 1 << 0;
const THREE: Rank = 1 << 1;
const FOUR: Rank = 1 << 2;
const FIVE: Rank = 1 << 3;
const SIX: Rank = 1 << 4;
const SEVEN: Rank = 1 << 5;
const EIGHT: Rank = 1 << 6;
const NINE: Rank = 1 << 7;
const TEN: Rank = 1 << 8;
const JACK: Rank = 1 << 9;
const QUEEN: Rank = 1 << 10;
const KING: Rank = 1 << 11;
const ACE: Rank = 1 << 12;

const BLACK_JOKER: Rank = 1 << 13;
const RED_JOKER: Rank = 1 << 14;
const EXTRA_JOKER: Rank = 1 << 15;

const PLAINS: Rank = TWO | THREE | FOUR | FIVE | SIX | SEVEN | EIGHT | NINE;
const PICTURES: Rank = TEN | JACK | QUEEN | KING | ACE;
const JOKERS: Rank = BLACK_JOKER | RED_JOKER | EXTRA_JOKER;

const ALL_SUITS: Cards = 1 << SPADE | 1 << HEART | 1 << DIAMOND | 1 << CLUB;

fn suit_of(card: Cards) -> Suit {
    card.trailing_zeros() & !15
}

#[test]
fn test_suit_of() {
    assert_eq!(suit_of(ACE << SPADE), SPADE);
    assert_eq!(suit_of(QUEEN << HEART), HEART);
    assert_eq!(suit_of(NINE << DIAMOND), DIAMOND);
    assert_eq!(suit_of(THREE << CLUB), CLUB);
    assert_eq!(suit_of(BLACK_JOKER << HEART), HEART);
    assert_eq!(suit_of(RED_JOKER << DIAMOND), DIAMOND);
    assert_eq!(suit_of(EXTRA_JOKER << CLUB), CLUB);
}

fn rank_of(card: Cards) -> Rank {
    card >> suit_of(card)
}

#[test]
fn test_rank_of() {
    assert_eq!(rank_of(TWO << SPADE), TWO);
    assert_eq!(rank_of(TEN << HEART), TEN);
    assert_eq!(rank_of(KING << DIAMOND), KING);
    assert_eq!(rank_of(ACE << CLUB), ACE);
    assert_eq!(rank_of(BLACK_JOKER << HEART), BLACK_JOKER);
    assert_eq!(rank_of(RED_JOKER << DIAMOND), RED_JOKER);
    assert_eq!(rank_of(EXTRA_JOKER << CLUB), EXTRA_JOKER);
}

fn inverse(suit: Suit) -> Suit {
    48 - suit
}

#[test]
fn test_inverse() {
    assert_eq!(inverse(SPADE), CLUB);
    assert_eq!(inverse(HEART), DIAMOND);
    assert_eq!(inverse(DIAMOND), HEART);
    assert_eq!(inverse(CLUB), SPADE);
}

fn reversing_jacks(trump: Suit) -> Cards {
    JACK << (16 ^ trump) | JACK << (32 ^ trump)
}

#[test]
fn test_reverse_jacks() {
    assert_eq!(reversing_jacks(SPADE), JACK << HEART | JACK << DIAMOND);
    assert_eq!(reversing_jacks(HEART), JACK << SPADE | JACK << CLUB);
    assert_eq!(reversing_jacks(DIAMOND), JACK << SPADE | JACK << CLUB);
    assert_eq!(reversing_jacks(CLUB), JACK << HEART | JACK << DIAMOND);
}

fn available_cards(hand: Cards, lead: Cards) -> Cards {
    let jokers_in_hand = hand & JOKERS;
    if lead == 0 { hand | jokers_in_hand * ALL_SUITS }
    else if lead == THREE << CLUB && jokers_in_hand != 0 { jokers_in_hand }
    else { 
        let must_follow = hand & (PLAINS | PICTURES) << suit_of(lead);
        if must_follow != 0 { must_follow | jokers_in_hand }
        else { hand }
    }
}

#[test]
fn test_available_cards() {
    assert_eq!(available_cards(ACE << SPADE | QUEEN << HEART | THREE << CLUB, 0), ACE << SPADE | QUEEN << HEART | THREE << CLUB);
    assert_eq!(available_cards(ACE << SPADE | RED_JOKER | EXTRA_JOKER, 0), ACE << SPADE | (RED_JOKER | EXTRA_JOKER) * ALL_SUITS);
    assert_eq!(available_cards(ACE << SPADE | RED_JOKER | EXTRA_JOKER, THREE << CLUB), RED_JOKER | EXTRA_JOKER);
    assert_eq!(available_cards(ACE << SPADE | (JACK | ACE) << CLUB, THREE << CLUB), (JACK | ACE) << CLUB);
    assert_eq!(available_cards(ACE << SPADE | JACK << CLUB | BLACK_JOKER, KING << SPADE), ACE << SPADE | BLACK_JOKER);
    assert_eq!(available_cards(ACE << SPADE | JACK << CLUB | BLACK_JOKER, QUEEN << HEART), ACE << SPADE | JACK << CLUB | BLACK_JOKER);
}

fn is_first_trick(n_remaining: CardIndex, n_players: PlayerIndex) -> bool {
    n_remaining >= 50 - (50 % n_players) - n_players
}

#[test]
fn test_is_first_trick() {
    assert!(is_first_trick(45, 5));
    assert!(!is_first_trick(40, 5));
    assert!(!is_first_trick(0, 5));
    assert!(is_first_trick(44, 4));
    assert!(!is_first_trick(40, 4));
    assert!(!is_first_trick(0, 4));
}

fn trick_taker(trick: &[Cards], trump: Suit, first_trick: bool) -> PlayerIndex {
    let field = trick.iter().fold(0, |acc, &card| acc | card);
    let mighty = if trump != SPADE { ACE << SPADE } else { ACE << CLUB };
    let strongest = if field & mighty != 0 {
        if field & QUEEN << HEART != 0 { QUEEN << HEART } else { mighty }
    }
    else if field & JOKERS * ALL_SUITS != 0 {
       if field & EXTRA_JOKER * ALL_SUITS != 0 { EXTRA_JOKER * ALL_SUITS } else { (BLACK_JOKER | RED_JOKER) * ALL_SUITS }
    }
    else if field & JACK << trump != 0 {
        JACK << trump
    }
    else if field & JACK << inverse(trump) != 0 {
        JACK << inverse(trump)
    }
    else {
        let critical = if field & (PLAINS | PICTURES) << trump != 0 { trump } else { suit_of(trick[trick.len() - 1]) };
        let effective = field & (PLAINS | PICTURES) << critical;
        if !first_trick && effective == field && effective & TWO * ALL_SUITS != 0 { TWO << critical }
        else { 1 << (63 - effective.leading_zeros()) }
    };
    trick.iter().rev().rposition(|&card| card & strongest != 0).unwrap()
}

#[test]
fn test_trick_taker() {
    assert_eq!(trick_taker(&[QUEEN << HEART, ACE << SPADE, EXTRA_JOKER, JACK << DIAMOND, ACE << CLUB], HEART, false), 4);
    assert_eq!(trick_taker(&[QUEEN << HEART, ACE << SPADE, EXTRA_JOKER, JACK << DIAMOND, ACE << CLUB], SPADE, false), 4);
    assert_eq!(trick_taker(&[ACE << SPADE, EXTRA_JOKER, JACK << DIAMOND, ACE << CLUB], CLUB, false), 3);
    assert_eq!(trick_taker(&[ACE << SPADE, EXTRA_JOKER, JACK << DIAMOND, ACE << CLUB], SPADE, false), 0);
    assert_eq!(trick_taker(&[ACE << CLUB, RED_JOKER, BLACK_JOKER, EXTRA_JOKER], DIAMOND, false), 0);
    assert_eq!(trick_taker(&[JACK << HEART, RED_JOKER, BLACK_JOKER, JACK << DIAMOND], DIAMOND, false), 2);
    assert_eq!(trick_taker(&[ACE << SPADE, JACK << DIAMOND, BLACK_JOKER, RED_JOKER], SPADE, false), 1);
    assert_eq!(trick_taker(&[JACK << SPADE, JACK << CLUB, JACK << HEART, JACK << DIAMOND], SPADE, false), 3);
    assert_eq!(trick_taker(&[ACE << SPADE, JACK << CLUB, JACK << HEART, JACK << DIAMOND], SPADE, false), 2);
    assert_eq!(trick_taker(&[KING << SPADE, JACK << DIAMOND, ACE << HEART, ACE << SPADE, JACK << HEART], SPADE, false), 1);
    assert_eq!(trick_taker(&[QUEEN << HEART, TEN << HEART, ACE << HEART, TWO << HEART, THREE << HEART], HEART, false), 1);
    assert_eq!(trick_taker(&[QUEEN << HEART, TEN << HEART, ACE << HEART, TWO << HEART, THREE << HEART], HEART, true), 2);
    assert_eq!(trick_taker(&[TEN << SPADE, KING << CLUB, ACE << HEART, QUEEN << CLUB], DIAMOND, false), 2);
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Camp {
    NAPOLEONIC,
    ALLIED,
    UNSETTLED,
}

impl Camp {
    fn opposite_party(belonging: Self) -> Self {
        match belonging {
            Self::NAPOLEONIC => Self::ALLIED,
            Self::ALLIED => Self::NAPOLEONIC,
            _ => Self::UNSETTLED
        }
    }

    fn to_string(self) -> String {
        String::from(match self {
            Self::NAPOLEONIC => "ナポレオン",
            Self::ALLIED => "平民",
            _ => ""
        })
    }
}

fn judge(contract: Declaration, declarer: Declaration, defender: Declaration) -> Camp {
    if defender > 20 - contract || declarer == 20 && contract < 20 { Camp::ALLIED }
    else if declarer >= contract && (defender > 0 || contract == 20) { Camp::NAPOLEONIC }
    else { Camp::UNSETTLED }
}

#[test]
fn test_judge() {
    assert_eq!(judge(20, 19, 1), Camp::ALLIED);
    assert_eq!(judge(19, 20, 0), Camp::ALLIED);
    assert_eq!(judge(17, 17, 1), Camp::NAPOLEONIC);
    assert_eq!(judge(20, 20, 0), Camp::NAPOLEONIC);
    assert_eq!(judge(18, 17, 2), Camp::UNSETTLED);
}

fn solve_last_trick(hands: &[Cards], napoleon: PlayerIndex, adjutant: PlayerIndex, mut turn: PlayerIndex, rotation: PlayerIndex, trump: Suit, contract: Declaration, declarer: Declaration, defender: Declaration) -> (Camp, Vec<Cards>, u64, u64) {
    let n_players = hands.len();
    let mut pv = Vec::with_capacity(N_CARDS);
    for _ in 0 .. n_players {
        turn = (turn + n_players - rotation) % n_players;
        pv.push(hands[turn]);
    }
    turn = (turn + trick_taker(&pv, trump, false) * rotation) % n_players;
    let result = if turn == napoleon || turn == adjutant { judge(contract, 20 - defender, defender) } else { judge(contract, declarer, 20 - declarer) };
    (result, pv, 1, 1)
}

// TODO: 手札のカードの強さが連続している場合、片方のみを探索するようにする
fn solve(hands: &mut [Cards], napoleon: PlayerIndex, adjutant: PlayerIndex, mut turn: PlayerIndex, mut rotation: PlayerIndex, variation: &mut [Cards], n_remaining: CardIndex, trump: Suit, contract: Declaration, mut declarer: Declaration, mut defender: Declaration) -> (Camp, Vec<Cards>, u64, u64) {
    let n_players = hands.len();
    let remainder = n_remaining % n_players;
    if remainder == 0 && n_remaining < variation.len() {
        let trick = &variation[n_remaining .. n_remaining + n_players];
        let field = trick.iter().fold(0, |acc, &card| acc | card);
        let n_picture_cards = (field & PICTURES * ALL_SUITS).count_ones() as Declaration;
        turn = (turn + trick_taker(trick, trump, is_first_trick(n_remaining, n_players)) * rotation) % n_players;
        if turn == napoleon || turn == adjutant { declarer += n_picture_cards; } else { defender += n_picture_cards; }
        let winner = judge(contract, declarer, defender);
        if winner != Camp::UNSETTLED {
            return (winner, Vec::with_capacity(variation.len() - n_remaining), 1, 1);
        }
        if (field & reversing_jacks(trump)).count_ones() == 1 { rotation = n_players - rotation; }
        if n_remaining == n_players {
            return solve_last_trick(&hands, napoleon, adjutant, turn, rotation, trump, contract, declarer, defender);
        }
    }
    let (mut n_visited_leaves, mut disproof_number) = (0, 0);
    let (mut toughest_variation, mut greatest_proof_number) = (Vec::new(), 0);
    let belonging = if turn == napoleon || turn == adjutant { Camp::NAPOLEONIC } else { Camp::ALLIED };
    let lead = if remainder == 0 { 0 } else { variation[n_remaining - remainder + n_players - 1] };
    let mut choice = available_cards(hands[turn], lead);
    while choice != 0 {
        let card = choice & !(choice - 1);
        let card_original = if card & JOKERS * ALL_SUITS == 0 { card } else { rank_of(card) };
        hands[turn] ^= card_original;
        variation[n_remaining - 1] = card;
        let (result, mut pv, n_subtree_leaves, proof_number) = solve(hands, napoleon, adjutant, (turn + rotation) % n_players, rotation, variation, n_remaining - 1, trump, contract, declarer, defender);
        hands[turn] ^= card_original;
        pv.push(card);
        n_visited_leaves += n_subtree_leaves;
        disproof_number += proof_number;
        if result == belonging {
            return (belonging, pv, n_visited_leaves, proof_number);
        }
        if proof_number > greatest_proof_number {
            toughest_variation = pv;
            greatest_proof_number = proof_number;
        }
        choice ^= card;
    }
    return (Camp::opposite_party(belonging), toughest_variation, n_visited_leaves, disproof_number);
}

#[test]
fn test_solve() {
    assert_eq!(solve(&mut [
        NINE << DIAMOND | (SIX | SEVEN | NINE | TEN) << CLUB | EXTRA_JOKER,
        JACK << SPADE | (FIVE | QUEEN) << HEART | (TEN | QUEEN) << DIAMOND | RED_JOKER,
        (THREE | SIX | ACE) << HEART | FOUR << DIAMOND | (THREE | KING) << CLUB,
        (FIVE | EIGHT | NINE | ACE) << SPADE | (JACK | ACE) << DIAMOND,
    ], 0, 3, 0, 1, &mut [0; 24], 24, CLUB, 17, 10, 0).0, Camp::NAPOLEONIC);
}

fn move_and_solve(hands_original: &[Cards], napoleon: PlayerIndex, adjutant: PlayerIndex, mut turn: PlayerIndex, mut rotation: PlayerIndex, variation: &mut [Cards], mut n_remaining: CardIndex, trump: Suit, contract: Declaration, mut declarer: Declaration, mut defender: Declaration, moves: &[Cards]) -> (Camp, Vec<Cards>, u64, u64) {
    let mut hands = Vec::from(hands_original);
    let n_players = hands.len();
    for &card in moves {
         let card_original = if card & JOKERS * ALL_SUITS == 0 { card } else { rank_of(card) };
        if hands[turn] & card_original == 0 { panic!("Invalid move"); }
        hands[turn] ^= card_original;
        turn = (turn + rotation) % n_players;
        n_remaining -= 1;
        variation[n_remaining] = card;
        if n_remaining % n_players == 0 && n_remaining < variation.len() {
            let trick = &variation[n_remaining .. n_remaining + n_players];
            let field = trick.iter().fold(0, |acc, &card| acc | card);
            let n_picture_cards = (field & PICTURES * ALL_SUITS).count_ones() as Declaration;
            turn = (turn + trick_taker(trick, trump, is_first_trick(n_remaining, n_players)) * rotation) % n_players;
            if turn == napoleon || turn == adjutant { declarer += n_picture_cards; } else { defender += n_picture_cards; }
            let winner = judge(contract, declarer, defender);
            if winner != Camp::UNSETTLED { return (winner, vec![0; 0], 1, 1); }
            if (field & reversing_jacks(trump)).count_ones() == 1 { rotation = n_players - rotation; }
        }
    }
    solve(&mut hands, napoleon, adjutant, turn, rotation, &mut variation[.. ((n_remaining - 1) / n_players + 1) * n_players], n_remaining, trump, contract, declarer, defender)
}

fn main() {
    let time_measurement_base = std::time::Instant::now();
    let mut hands = [
        (TWO | QUEEN) << SPADE | (SIX) << DIAMOND | (TWO | THREE | FIVE | NINE) << CLUB | BLACK_JOKER,
        (SIX | SEVEN | NINE) << HEART | (EIGHT | ACE) << DIAMOND | (FOUR | JACK | KING) << CLUB,
        (THREE | SIX | TEN | KING) << SPADE | (FOUR | FIVE) << DIAMOND | (TEN | QUEEN) << CLUB,
        (JACK) << HEART | (THREE | JACK | QUEEN) << DIAMOND | (EIGHT | ACE) << CLUB | RED_JOKER | EXTRA_JOKER,
    ];
    let napoleon = 3;
    let adjutant = 2;
    let turn = 0;
    let rotation = 1;
    let mut variation = [0; N_CARDS];
    let n_remaining = hands.iter().fold(0, |acc, &hand| acc | hand).count_ones() as CardIndex;
    let trump = DIAMOND;
    let contract = 17;
    let declarer = 7;
    let defender = 1;
    let n_players = hands.len();
    let (result, pv, n_visited_leaves, proof_number) = solve(&mut hands, napoleon, adjutant, turn, rotation, &mut variation[.. ((n_remaining - 1) / n_players + 1) * n_players], n_remaining, trump, contract, declarer, defender);
    println!("{} wins", result.to_string());
    println!("{}", pv.iter().rev().map(|&x| card_into_string(x)).collect::<Vec<_>>().join(" "));
    println!("{} leaves visited", n_visited_leaves);
    println!("{} is an upper bound of the proof number", proof_number);
    let time_elapsed = time_measurement_base.elapsed();
    println!("{}.{:03} sec", time_elapsed.as_secs(), time_elapsed.subsec_nanos() / 1_000_000);
}

#[allow(dead_code)]
fn card_into_string(card: Cards) -> String {
    let (suit, rank) = (suit_of(card), rank_of(card));
    let suit_name = match suit {
        SPADE => "♠",
        HEART => "♥",
        DIAMOND => "♦",
        CLUB => "♣",
        _ => ""
    };
    let rank_name = match rank {
        TWO => "2",
        THREE => "3",
        FOUR => "4",
        FIVE => "5",
        SIX => "6",
        SEVEN => "7",
        EIGHT => "8",
        NINE => "9",
        TEN => "10",
        JACK => "J",
        QUEEN => "Q",
        KING => "K",
        ACE => "A",
        BLACK_JOKER => "黒ジョーカー",
        RED_JOKER => "赤ジョーカー",
        EXTRA_JOKER => "エクストラ",
        _ => ""
    };
    String::from(suit_name) + rank_name
}

#[allow(dead_code)]
fn cards_into_string(mut cards: Cards) -> String {
    let mut cards_string = String::default();
    while cards != 0 {
        let card = cards & !(cards - 1);
        cards_string += card_into_string(card).as_str();
        cards ^= card;
        if cards != 0 { cards_string += ","; }
    }
    cards_string
}
