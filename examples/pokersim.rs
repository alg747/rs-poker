// put in rs-poker/examples/ and run with `cargo run --example pokersim

extern crate rs_poker;
use rs_poker::core::{Card, Hand, Rankable};
use rs_poker::holdem::MonteCarloGame;

fn main() {
    let sim = 100_000;
    let mut hands: Vec<Hand> = ["askd", "7d6d"]
        .iter()
        .map(|s| Hand::new_from_str(s).unwrap())
        .collect();

    // let flop: [&str; 0] = [];
    let flop = ["8d", "2c", "js"];
    let turn_card = Card::try_from("").ok();
    let river_card = Card::try_from("").ok();

    println!("\n---------------PREFLOP ------------");
    for h in hands.iter_mut() {
        println!("{:?} has:\t{:?}", h, h.rank());
    }
    let preflop_eq = MonteCarloGame::new(hands.clone())
        .unwrap()
        .estimate_equity(sim);
    println!("\n\t\t\t\tPreflop equity:\t{:?}\n", preflop_eq);

    let mut board: Vec<Card> = vec![];
    if !flop.is_empty() {
        println!("---------------AT THE FLOP ------------");
        board.extend(flop.iter().map(|&s| Card::try_from(s).unwrap()));
        for h in hands.iter_mut() {
            for c in &board {
                (*h).insert(*c);
            }
            println!("{:?} has:\t\t--> {:?}", h, h.rank())
        }

        let flop_eq = MonteCarloGame::new(hands.clone())
            .unwrap()
            .estimate_equity(sim);
        println!("\n\t\t\t\tFlop equity :\t{:?}\n", flop_eq);

        if let Some(c) = turn_card {
            println!("---------------AT THE TURN ------------");
            for h in hands.iter_mut() {
                (*h).insert(c);
                println!("{:?} has:\t--> {:?}", h, h.rank())
            }

            let turn_eq = MonteCarloGame::new(hands.clone())
                .unwrap()
                .estimate_equity(sim);

            println!("\n\t\t\t\tTurn equity:\t{:?}\n", turn_eq);

            if let Some(c) = river_card {
                println!("---------------AT THE RIVER ------------");
                for h in hands.iter_mut() {
                    (*h).insert(c);
                    println!("{:?} has: --> {:?}", h, h.rank())
                }

                let river_eq = MonteCarloGame::new(hands.clone())
                    .unwrap()
                    .estimate_equity(sim);

                println!("\n\t\t\t\tRiver equity:\t{:?}\n", river_eq);
            }
        }
    }
    // let hand = Hand::new_from_str("Ad8h9cTc5c").unwrap();
}
