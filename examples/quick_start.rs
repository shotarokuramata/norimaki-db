/// Quick Start Example for Norimaki DB
/// 
/// This example shows the most basic usage patterns.
/// Run with: cargo run --example quick_start

use norimaki_db::{
    BoatRaceEngine, MemoryStore, MonthlySchedule, RaceEvent, 
    generate_tournament_id, Result
};
use serde::{Serialize, Deserialize};

// Custom race data structure
#[derive(Debug, Serialize, Deserialize)]
struct SimpleRace {
    race_number: u32,
    start_time: String,
    winner: Option<String>,
}

fn main() -> Result<()> {
    println!("🚤 Norimaki DB Quick Start");
    println!("=========================\n");

    // 1. Create engine
    let store = MemoryStore::new();
    let mut engine = BoatRaceEngine::new(store);
    println!("✅ Created engine with in-memory storage");

    // 2. Create and save monthly schedule
    let schedule = MonthlySchedule {
        year_month: "2025-09".to_string(),
        events: vec![
            RaceEvent {
                venue_id: 4,
                venue_name: "平和島".to_string(),
                event_name: "トーキョー・ベイ・カップ".to_string(),
                grade: "G1".to_string(),
                start_date: "2025-09-10".to_string(),
                duration_days: 3,
            },
        ],
    };
    
    engine.put_monthly_schedule(&schedule)?;
    println!("✅ Saved monthly schedule for September 2025");

    // 3. Retrieve monthly schedule
    let retrieved = engine.get_monthly_schedule(202509)?;
    println!("📅 Retrieved schedule: {} ({} events)", 
             retrieved.year_month, retrieved.events.len());
    
    for event in &retrieved.events {
        println!("   • {} - {}", event.venue_name, event.event_name);
    }

    // 4. Add race data
    let tournament_id = generate_tournament_id("平和島", "トーキョー・ベイ・カップ");
    
    let race1 = SimpleRace {
        race_number: 1,
        start_time: "14:00".to_string(),
        winner: Some("選手A".to_string()),
    };
    
    let race2 = SimpleRace {
        race_number: 2,
        start_time: "14:30".to_string(),
        winner: None, // Race not finished yet
    };

    let timestamp1 = 1694524800000;
    let timestamp2 = 1694526600000; // 30 minutes later

    engine.put_race_data(&tournament_id, timestamp1, &race1)?;
    engine.put_race_data(&tournament_id, timestamp2, &race2)?;
    println!("✅ Added 2 races to the tournament");

    // 5. Retrieve race data
    let retrieved_race: SimpleRace = engine.get_race_data(&tournament_id, timestamp1)?;
    println!("🏁 Race {}: {} - Winner: {:?}", 
             retrieved_race.race_number, 
             retrieved_race.start_time,
             retrieved_race.winner);

    // 6. Get all races for the tournament
    let all_races: Vec<SimpleRace> = engine.get_tournament_races(&tournament_id)?;
    println!("📊 Total races in tournament: {}", all_races.len());

    // 7. Show statistics
    let (monthly_count, tournament_count, race_count) = engine.get_statistics()?;
    println!("\n📈 Database Statistics:");
    println!("   Monthly entries: {}", monthly_count);
    println!("   Tournaments: {}", tournament_count);
    println!("   Races: {}", race_count);

    println!("\n🎉 Quick start complete!");
    Ok(())
}