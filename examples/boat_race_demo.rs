/// 競艇データエンジンのデモンストレーション
/// 
/// 使用方法: cargo run --example boat_race_demo

use norimaki_db::{
    BoatRaceEngine, MemoryStore, FileStore, MonthlySchedule, RaceEvent, 
    Result, generate_tournament_id
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RaceData {
    race_number: u32,
    start_time: String,
    participants: Vec<Participant>,
    weather: String,
    wind_speed: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Participant {
    racer_id: u32,
    name: String,
    boat_number: u32,
    engine_number: u32,
}

fn main() -> Result<()> {
    println!("🚤 競艇データエンジン デモンストレーション");
    println!("=========================================\n");

    // デモ1: メモリストレージでの基本操作
    demo_memory_operations()?;
    
    // デモ2: ファイルストレージでの永続化
    demo_file_operations()?;
    
    // デモ3: 月跨ぎ大会の処理
    demo_cross_month_tournament()?;

    println!("🎉 全てのデモが正常に完了しました！");
    Ok(())
}

fn demo_memory_operations() -> Result<()> {
    println!("📝 デモ1: メモリストレージでの基本操作");
    println!("----------------------------------------");

    let store = MemoryStore::new();
    let mut engine = BoatRaceEngine::new(store);

    // 1. 月別スケジュールの作成と保存
    let schedule = create_sample_schedule();
    println!("📅 2025年9月のスケジュールを保存中...");
    engine.put_monthly_schedule(&schedule)?;

    // 2. 月別スケジュールの取得
    println!("📋 2025年9月のスケジュールを取得中...");
    let retrieved = engine.get_monthly_schedule(202509)?;
    println!("✅ 取得完了: {} ({} 大会)", retrieved.year_month, retrieved.events.len());
    
    for (i, event) in retrieved.events.iter().enumerate() {
        println!("  {}. {} - {} ({}日間, {})",
            i + 1, event.venue_name, event.event_name, event.duration_days, event.grade);
    }

    // 3. 個別レースデータの保存
    println!("\n🏁 個別レースデータの保存...");
    let tokyo_bay_cup_id = generate_tournament_id("平和島", "トーキョー・ベイ・カップ");
    
    for race_num in 1..=3 {
        let race_data = create_sample_race_data(race_num);
        let timestamp = 1694524800000 + (race_num as u64 * 3600000); // 1時間間隔
        engine.put_race_data(&tokyo_bay_cup_id, timestamp, &race_data)?;
        println!("  レース{}を保存: {}", race_num, race_data.start_time);
    }

    // 4. レースデータの取得
    println!("\n📊 レースデータの取得...");
    let races: Vec<RaceData> = engine.get_tournament_races(&tokyo_bay_cup_id)?;
    println!("✅ {}レース取得完了", races.len());
    
    for race in &races {
        println!("  R{}: {} (参加者{}名, 天候: {})",
            race.race_number, race.start_time, race.participants.len(), race.weather);
    }

    // 5. 統計情報の表示
    println!("\n📈 データ統計:");
    let (monthly_count, tournament_count, race_count) = engine.get_statistics()?;
    println!("  月別エントリ: {}", monthly_count);
    println!("  大会数: {}", tournament_count);
    println!("  レース数: {}", race_count);

    println!("\n✅ デモ1完了\n");
    Ok(())
}

fn demo_file_operations() -> Result<()> {
    println!("💾 デモ2: ファイルストレージでの永続化");
    println!("----------------------------------------");

    let db_file = "demo_boat_race.json";
    
    // ファイルが既に存在する場合は削除
    let _ = std::fs::remove_file(db_file);

    {
        // 1. データ保存
        let store = FileStore::new(db_file)?;
        let mut engine = BoatRaceEngine::new(store);
        
        println!("💾 ファイルストレージにデータを保存中...");
        let schedule = create_sample_schedule();
        engine.put_monthly_schedule(&schedule)?;
        
        let tournament_id = generate_tournament_id("桐生", "群馬クレインサンダーズカップ");
        let race_data = create_sample_race_data(1);
        engine.put_race_data(&tournament_id, 1694524800000, &race_data)?;
        
        println!("✅ データ保存完了");
    }

    {
        // 2. データ読み込み
        println!("📖 ファイルから데이터を読み込み中...");
        let store = FileStore::new(db_file)?;
        let mut engine = BoatRaceEngine::new(store);
        
        let schedule = engine.get_monthly_schedule(202509)?;
        println!("✅ 月別スケジュール読み込み完了: {} 大会", schedule.events.len());
        
        let tournament_id = generate_tournament_id("桐生", "群馬クレインサンダーズカップ");
        let races: Vec<RaceData> = engine.get_tournament_races(&tournament_id)?;
        println!("✅ レースデータ読み込み完了: {} レース", races.len());
    }

    // クリーンアップ
    let _ = std::fs::remove_file(db_file);
    println!("🗑️ テンポラリファイルをクリーンアップ");
    
    println!("\n✅ デモ2完了\n");
    Ok(())
}

fn demo_cross_month_tournament() -> Result<()> {
    println!("🗓️ デモ3: 月跨ぎ大会の処理");
    println!("----------------------------------------");

    let store = MemoryStore::new();
    let mut engine = BoatRaceEngine::new(store);

    // 年末年始に跨る大会を作成
    let year_end_tournament = RaceEvent {
        venue_id: 24,
        venue_name: "大村".to_string(),
        event_name: "年末年始特別競走".to_string(),
        grade: "SG".to_string(),
        start_date: "2025-12-28".to_string(),
        duration_days: 8, // 2026-01-04まで
    };

    println!("🎊 年末年始大会を複数月に登録中...");
    println!("  期間: {} ～ {} ({} 日間)",
        year_end_tournament.start_date,
        chrono::NaiveDate::parse_from_str(&year_end_tournament.start_date, "%Y-%m-%d")
            .unwrap()
            .checked_add_signed(chrono::Duration::days(year_end_tournament.duration_days as i64 - 1))
            .unwrap()
            .format("%Y-%m-%d"),
        year_end_tournament.duration_days
    );

    engine.register_tournament_to_months(&year_end_tournament)?;

    // 12月のスケジュールを確認
    println!("\n📅 2025年12月のスケジュール:");
    let dec_schedule = engine.get_monthly_schedule(202512)?;
    for event in &dec_schedule.events {
        println!("  • {} - {} ({})",
            event.venue_name, event.event_name, event.grade);
    }

    // 1月のスケジュールを確認
    println!("\n📅 2026年1月のスケジュール:");
    let jan_schedule = engine.get_monthly_schedule(202601)?;
    for event in &jan_schedule.events {
        println!("  • {} - {} ({})",
            event.venue_name, event.event_name, event.grade);
    }

    println!("\n✅ 月跨ぎ大会が両方の月に正しく登録されました");
    println!("✅ デモ3完了\n");
    
    Ok(())
}

fn create_sample_schedule() -> MonthlySchedule {
    MonthlySchedule {
        year_month: "2025-09".to_string(),
        events: vec![
            RaceEvent {
                venue_id: 1,
                venue_name: "桐生".to_string(),
                event_name: "バスケで群馬を熱くする群馬クレインサンダーズカップ".to_string(),
                grade: "一般".to_string(),
                start_date: "2025-09-11".to_string(),
                duration_days: 6,
            },
            RaceEvent {
                venue_id: 4,
                venue_name: "平和島".to_string(),
                event_name: "開設７１周年記念トーキョー・ベイ・カップ".to_string(),
                grade: "G1".to_string(),
                start_date: "2025-09-10".to_string(),
                duration_days: 7,
            },
            RaceEvent {
                venue_id: 12,
                venue_name: "住之江".to_string(),
                event_name: "第５３回高松宮記念特別競走".to_string(),
                grade: "G1".to_string(),
                start_date: "2025-09-13".to_string(),
                duration_days: 6,
            },
        ],
    }
}

fn create_sample_race_data(race_number: u32) -> RaceData {
    RaceData {
        race_number,
        start_time: format!("2025-09-10 {}:00:00", 10 + race_number),
        participants: vec![
            Participant {
                racer_id: 4001,
                name: "山田太郎".to_string(),
                boat_number: 1,
                engine_number: 15,
            },
            Participant {
                racer_id: 4002,
                name: "佐藤花子".to_string(),
                boat_number: 2,
                engine_number: 23,
            },
            Participant {
                racer_id: 4003,
                name: "田中次郎".to_string(),
                boat_number: 3,
                engine_number: 31,
            },
        ],
        weather: "晴れ".to_string(),
        wind_speed: 2.5,
    }
}