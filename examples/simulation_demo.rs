use rand::{SeedableRng, rngs::StdRng};
use rs_poker::arena::{
    Agent, GameState, HoldemSimulationBuilder,
    action::Action,
    agent::{CallingAgent, FoldingAgent, RandomAgent, RandomPotControlAgent},
    historian::{FnHistorian, VecHistorian},
};

fn main() {
    // Initialize tracing for detailed logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .init();

    println!("=== 5-Player Texas Hold'em Simulation Demo ===\n");

    // Use a seeded RNG for reproducible results
    let mut rng = StdRng::seed_from_u64(42);

    // Create 5 players with different starting stacks
    let stacks = vec![1000.0, 800.0, 1200.0, 900.0, 1100.0];
    let game_state = GameState::new_starting(
        stacks.clone(),
        20.0, // big blind
        10.0, // small blind
        0.0,  // ante
        0,    // dealer position
    );

    // Create different types of agents with distinct playing styles
    let agents: Vec<Box<dyn Agent>> = vec![
        // Player 0: Conservative random agent (folds often)
        Box::new(RandomAgent::new(
            vec![0.4, 0.5, 0.7], // fold percentages by raise count
            vec![0.3, 0.2, 0.1], // call percentages by raise count
        )),
        // Player 1: Calling station (calls almost everything)
        Box::new(CallingAgent),
        // Player 2: Aggressive random agent (raises often)
        Box::new(RandomAgent::new(
            vec![0.1, 0.2, 0.3], // fold percentages
            vec![0.4, 0.3, 0.2], // call percentages
        )),
        // Player 3: Pot control agent (calculated play)
        Box::new(RandomPotControlAgent::new(vec![0.6, 0.4, 0.2])),
        // Player 4: Tight folder (folds most hands)
        Box::new(FoldingAgent),
    ];

    // Create historians for detailed logging
    let vec_historian = Box::new(VecHistorian::new());
    let storage = vec_historian.get_storage();

    // Create a custom function historian for real-time action logging
    let fn_historian = Box::new(FnHistorian::new(|_id, _game_state, action| {
        log_action(&action);
        Ok(())
    }));

    let historians: Vec<Box<dyn rs_poker::arena::historian::Historian>> =
        vec![vec_historian, fn_historian];

    // Build the simulation
    let mut sim = HoldemSimulationBuilder::default()
        .game_state(game_state)
        .agents(agents)
        .historians(historians)
        .build()
        .unwrap();

    println!("Starting stacks:");
    for (i, stack) in stacks.iter().enumerate() {
        println!("  Player {i}: ${stack:.2}");
    }
    println!("\nBlinds: ${:.2}/${:.2}", 10.0, 20.0);
    println!("Simulation ID: {}\n", sim.id);

    // Run the simulation
    println!("🎰 Starting game...\n");
    sim.run(&mut rng);

    // Print final results
    println!("\n=== FINAL RESULTS ===");
    println!("Final stacks:");
    for (i, stack) in sim.game_state.stacks.iter().enumerate() {
        let change = stack - stacks[i];
        let change_str = if change >= 0.0 {
            format!("+${change:.2}")
        } else {
            format!("-${:.2}", change.abs())
        };
        println!("  Player {i}: ${stack:.2} ({change_str})");
    }

    println!("\nPlayer winnings:");
    for (i, winnings) in sim.game_state.player_winnings.iter().enumerate() {
        println!("  Player {i}: ${winnings:.2}");
    }

    // Print detailed action history
    println!("\n=== DETAILED ACTION HISTORY ===");
    let history = storage.borrow();
    for (i, record) in history.iter().enumerate() {
        println!("\n{}. {:?}", i + 1, record.action);

        // Show key game state information for important actions
        match &record.action {
            Action::PlayedAction(payload) => {
                println!(
                    "   Player {} stack: ${:.2} -> ${:.2}",
                    payload.idx,
                    payload.player_stack + get_action_amount(&payload.action),
                    payload.player_stack
                );
            }
            Action::Award(payload) => {
                println!(
                    "   Player {} awarded ${:.2} from pot of ${:.2}",
                    payload.idx, payload.award_amount, payload.total_pot
                );
                if let Some(rank) = payload.rank {
                    println!("   Winning hand: {rank:?}");
                }
            }
            Action::DealCommunity(card) => {
                println!("   Community card: {card}");
            }
            _ => {}
        }
    }

    println!("\n=== GAME STATISTICS ===");
    println!("Total actions logged: {}", history.len());
    println!("Final round: {:?}", sim.game_state.round);
    println!(
        "Total pot distributed: ${:.2}",
        sim.game_state.player_winnings.iter().sum::<f32>()
    );
}

fn log_action(action: &Action) {
    match action {
        Action::GameStart(payload) => {
            println!(
                "🎮 Game started - BB: ${:.2}, SB: ${:.2}, Ante: ${:.2}",
                payload.big_blind, payload.small_blind, payload.ante
            );
        }
        Action::PlayerSit(payload) => {
            println!(
                "💺 Player {} sits with ${:.2}",
                payload.idx, payload.player_stack
            );
        }
        Action::DealStartingHand(payload) => {
            println!("🃏 Dealt {} to Player {}", payload.card, payload.idx);
        }
        Action::RoundAdvance(round) => {
            println!("🔄 Round advance to: {round:?}");
        }
        Action::PlayedAction(payload) => {
            let action_str = match payload.action {
                rs_poker::arena::action::AgentAction::Fold => "folds".to_string(),
                rs_poker::arena::action::AgentAction::Call => "calls".to_string(),
                rs_poker::arena::action::AgentAction::Bet(amount) => format!("bets ${amount:.2}"),
                rs_poker::arena::action::AgentAction::AllIn => "goes all-in".to_string(),
            };
            println!(
                "🎯 Player {} {} (stack: ${:.2})",
                payload.idx, action_str, payload.player_stack
            );
        }
        Action::FailedAction(payload) => {
            println!(
                "❌ Player {} failed action {:?} -> {:?}",
                payload.result.idx, payload.action, payload.result.action
            );
        }
        Action::ForcedBet(payload) => {
            let bet_type = match payload.forced_bet_type {
                rs_poker::arena::action::ForcedBetType::Ante => "ante",
                rs_poker::arena::action::ForcedBetType::SmallBlind => "small blind",
                rs_poker::arena::action::ForcedBetType::BigBlind => "big blind",
            };
            println!(
                "💰 Player {} posts {} ${:.2}",
                payload.idx, bet_type, payload.bet
            );
        }
        Action::DealCommunity(card) => {
            println!("🌟 Community card: {card}");
        }
        Action::Award(payload) => {
            println!(
                "🏆 Player {} wins ${:.2} from ${:.2} pot",
                payload.idx, payload.award_amount, payload.total_pot
            );
            if let Some(rank) = payload.rank {
                println!("    Winning hand: {rank:?}");
            }
        }
    }
}

fn get_action_amount(action: &rs_poker::arena::action::AgentAction) -> f32 {
    match action {
        rs_poker::arena::action::AgentAction::Bet(amount) => *amount,
        rs_poker::arena::action::AgentAction::Call => 0.0, // Amount varies
        rs_poker::arena::action::AgentAction::Fold => 0.0,
        rs_poker::arena::action::AgentAction::AllIn => 0.0, // Amount varies
    }
}
