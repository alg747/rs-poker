use rand::{SeedableRng, rngs::StdRng};
use rs_poker::arena::{
    Agent, GameState, HoldemSimulationBuilder,
    action::Action,
    agent::{CallingAgent, RandomAgent},
    historian::FnHistorian,
};

fn main() {
    println!("=== Simple 5-Player Texas Hold'em Demo ===\n");

    // Use a seeded RNG for reproducible results
    let mut rng = StdRng::seed_from_u64(12345);

    // Create 5 players with smaller stacks for quicker games
    let stacks = vec![200.0, 200.0, 200.0, 200.0, 200.0];
    let game_state = GameState::new_starting(
        stacks.clone(),
        10.0, // big blind
        5.0,  // small blind
        0.0,  // ante
        0,    // dealer position
    );

    // Create 5 different agents
    let agents: Vec<Box<dyn Agent>> = vec![
        Box::new(RandomAgent::new(vec![0.2], vec![0.6])), // Loose player
        Box::new(CallingAgent),                           // Calling station
        Box::new(RandomAgent::new(vec![0.5], vec![0.3])), // Tight player
        Box::new(RandomAgent::new(vec![0.3], vec![0.4])), // Balanced player
        Box::new(RandomAgent::new(vec![0.6], vec![0.2])), // Very tight player
    ];

    // Create a simple logging historian
    let fn_historian = Box::new(FnHistorian::new(|_id, _game_state, action| {
        print_action(&action);
        Ok(())
    }));

    let historians: Vec<Box<dyn rs_poker::arena::historian::Historian>> = vec![fn_historian];

    // Build the simulation
    let mut sim = HoldemSimulationBuilder::default()
        .game_state(game_state)
        .agents(agents)
        .historians(historians)
        .build()
        .unwrap();

    println!("Starting stacks: ${:.0} each", stacks[0]);
    println!("Blinds: ${:.0}/${:.0}\n", 5.0, 10.0);

    // Run the simulation
    sim.run(&mut rng);

    // Print final results
    println!("\n=== FINAL RESULTS ===");
    for (i, stack) in sim.game_state.stacks.iter().enumerate() {
        let change = stack - stacks[i];
        let status = if change > 0.0 {
            "📈"
        } else if change < 0.0 {
            "📉"
        } else {
            "➖"
        };
        println!("Player {i}: ${stack:.0} ({change:+.0}) {status}");
    }

    let total_winnings: f32 = sim.game_state.player_winnings.iter().sum();
    println!("\nTotal pot distributed: ${total_winnings:.0}");
}

fn print_action(action: &Action) {
    match action {
        Action::GameStart(_) => {
            println!("🎮 Game started!");
        }
        Action::PlayerSit(payload) => {
            println!(
                "💺 Player {} joins with ${:.0}",
                payload.idx, payload.player_stack
            );
        }
        Action::DealStartingHand(payload) => {
            println!("🃏 Player {} receives {}", payload.idx, payload.card);
        }
        Action::RoundAdvance(round) => {
            println!("\n🔄 --- {round:?} ---");
        }
        Action::PlayedAction(payload) => {
            let action_str = match payload.action {
                rs_poker::arena::action::AgentAction::Fold => "folds 😔".to_string(),
                rs_poker::arena::action::AgentAction::Call => "calls 📞".to_string(),
                rs_poker::arena::action::AgentAction::Bet(amount) => {
                    if amount > payload.starting_bet {
                        format!("raises to ${amount:.0} 🚀")
                    } else {
                        format!("bets ${amount:.0} 💰")
                    }
                }
                rs_poker::arena::action::AgentAction::AllIn => "goes ALL-IN! 🎯".to_string(),
            };
            println!(
                "   Player {} {} (stack: ${:.0})",
                payload.idx, action_str, payload.player_stack
            );
        }
        Action::ForcedBet(payload) => {
            let bet_type = match payload.forced_bet_type {
                rs_poker::arena::action::ForcedBetType::SmallBlind => "small blind",
                rs_poker::arena::action::ForcedBetType::BigBlind => "big blind",
                _ => "ante",
            };
            println!(
                "   Player {} posts {} ${:.0}",
                payload.idx, bet_type, payload.bet
            );
        }
        Action::DealCommunity(card) => {
            println!("🌟 Community: {card}");
        }
        Action::Award(payload) => {
            println!(
                "🏆 Player {} wins ${:.0}!",
                payload.idx, payload.award_amount
            );
            if let Some(rank) = payload.rank {
                println!("   Winning hand: {rank:?}");
            }
        }
        Action::FailedAction(payload) => {
            println!("❌ Player {} invalid action -> folds", payload.result.idx);
        }
    }
}
