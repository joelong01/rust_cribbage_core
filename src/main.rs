#![allow(dead_code)]
use arrayvec::ArrayVec;
mod cards;
use rand::{thread_rng, Rng};
use strum::IntoEnumIterator;
use rand::seq::SliceRandom;


fn main() {
    let mut deck = ArrayVec::<cards::Card, 52>::new();
    for o in cards::Ordinal::iter() {
        for s in cards::Suit::iter() {
            let card = cards::Card::new(o, s);
            deck.push(card);
        }
    }

    shuffle(&mut deck);
    dump_deck(&deck);

    let player_cut = cut();
    let mut computer_cut = cut();
    //
    //  stranger things have happened than getting the same card twice
    while player_cut == computer_cut {
        computer_cut = cut();
    }

    println!(
        "\nComputer Cut: {}\nPlayer Cut: {}\n",
        deck[computer_cut], deck[player_cut]
    );

    println!(" Player Hand\t ComputerHand");
    println!("==============\t==============");
    let mut i = 0;
    let mut player_hand = Vec::new(); // these are vectors of references
    let mut computer_hand = Vec::new();

    while i < 12 {
        println!("{}\t{}", deck[i], deck[i + 1]);
        player_hand.push(&deck[i]);
      //  deck[i].set_owner(cards::Owner::Player);        
        computer_hand.push(&deck[i + 1]);
       // deck[i+1].set_owner(cards::Owner::Player);
        i += 2;
    }

    let shared_card  = &deck[12];

    println!("\n\nSorted Player Hand");
    println!("------------------");

    player_hand.sort_by_key(|a| a.ordinal());
    dump_hand(player_hand);

    println!("\n\nSorted Computer Hand");
    println!("--------------------");

    computer_hand.sort_by_key(|a| a.ordinal());
    dump_hand(computer_hand);

    println!("\n\nShared Card");
    println!("------------");
    println!("{}", shared_card);
}

//
//  return the index of the cut card
fn cut() -> usize {
    thread_rng().gen_range(0..51)
}

fn shuffle(deck: &mut ArrayVec<cards::Card, 52>) {
    let mut rng = thread_rng();
    deck.shuffle(&mut rng);
}

fn dump_hand(hand: Vec<&cards::Card>) {
    for card in hand {
        println!("{}\t", card);
        
    }
}

fn dump_deck(deck: &ArrayVec<cards::Card, 52>) {
    let mut count = 0;
    for card in deck {
        print!("{}\t", card);
        count += 1;
        if count % 4 == 0 {
            println!(" ");
        }
    }
}
