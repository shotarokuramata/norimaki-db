/// 競艇データエンジン
/// 
/// KeyValueStoreを基盤とした競艇データ専用の高級API

use crate::{
    key::{monthly_key, tournament_key, monthly_scan_range, tournament_scan_range, generate_tournament_id},
    value::{serialize_to_string, deserialize_from_string},
    KeyValueStore, Result, MonthlySchedule, RaceEvent,
};
use serde::{Serialize, de::DeserializeOwned};
use chrono::{NaiveDate, Datelike};

pub struct BoatRaceEngine<K: KeyValueStore> {
    store: K,
}

impl<K: KeyValueStore> BoatRaceEngine<K> {
    /// 新しいエンジンインスタンスを作成
    pub fn new(store: K) -> Self {
        Self { store }
    }

    /// ストアへの参照を取得
    pub fn store(&self) -> &K {
        &self.store
    }

    /// 月別スケジュールを保存
    /// 
    /// # Arguments
    /// * `schedule` - 保存する月別スケジュール
    /// 
    /// # Returns
    /// 操作結果
    pub fn put_monthly_schedule(&mut self, schedule: &MonthlySchedule) -> Result<()> {
        // 年月をu32に変換 (例: "2025-09" -> 202509)
        let year_month = parse_year_month(&schedule.year_month)?;
        
        for event in &schedule.events {
            let tournament_id = generate_tournament_id(&event.venue_name, &event.event_name);
            let key = monthly_key(year_month, &tournament_id);
            let value = serialize_to_string(event)?;
            self.store.put(key, value)?;
        }
        
        Ok(())
    }

    /// 月別スケジュールを取得
    /// 
    /// # Arguments
    /// * `year_month` - 取得対象の年月 (例: 202509)
    /// 
    /// # Returns
    /// 月別スケジュール
    pub fn get_monthly_schedule(&mut self, year_month: u32) -> Result<MonthlySchedule> {
        let (start, end) = monthly_scan_range(year_month);
        let results = self.store.scan(&start, &end)?;
        
        let mut events = Vec::new();
        for (_, value) in results {
            let event: RaceEvent = deserialize_from_string(&value)?;
            events.push(event);
        }
        
        // 開始日でソート
        events.sort_by(|a, b| a.start_date.cmp(&b.start_date));
        
        Ok(MonthlySchedule {
            year_month: format_year_month(year_month),
            events,
        })
    }

    /// 個別レースデータを保存
    /// 
    /// # Arguments
    /// * `tournament_id` - 大会ID
    /// * `timestamp` - レースのタイムスタンプ
    /// * `data` - レースデータ
    /// 
    /// # Returns
    /// 操作結果
    pub fn put_race_data<T: Serialize>(&mut self, tournament_id: &str, timestamp: u64, data: &T) -> Result<()> {
        let key = tournament_key(tournament_id, timestamp);
        let value = serialize_to_string(data)?;
        self.store.put(key, value)
    }

    /// 大会の全レースデータを取得
    /// 
    /// # Arguments
    /// * `tournament_id` - 大会ID
    /// 
    /// # Returns
    /// レースデータのベクター（タイムスタンプ順）
    pub fn get_tournament_races<T: DeserializeOwned>(&mut self, tournament_id: &str) -> Result<Vec<T>> {
        let (start, end) = tournament_scan_range(tournament_id);
        let results = self.store.scan(&start, &end)?;
        
        let mut races = Vec::new();
        for (_, value) in results {
            let race: T = deserialize_from_string(&value)?;
            races.push(race);
        }
        
        Ok(races)
    }

    /// 特定のレースデータを取得
    /// 
    /// # Arguments
    /// * `tournament_id` - 大会ID
    /// * `timestamp` - レースのタイムスタンプ
    /// 
    /// # Returns
    /// レースデータ
    pub fn get_race_data<T: DeserializeOwned>(&self, tournament_id: &str, timestamp: u64) -> Result<T> {
        let key = tournament_key(tournament_id, timestamp);
        let value = self.store.get(&key)?
            .ok_or_else(|| crate::StoreError::NotFound)?;
        deserialize_from_string(&value)
    }

    /// 大会を複数の月に登録（月跨ぎ大会対応）
    /// 
    /// # Arguments
    /// * `tournament` - 登録する大会情報
    /// 
    /// # Returns
    /// 操作結果
    pub fn register_tournament_to_months(&mut self, tournament: &RaceEvent) -> Result<()> {
        let start_date = NaiveDate::parse_from_str(&tournament.start_date, "%Y-%m-%d")
            .map_err(|e| crate::StoreError::InvalidValue)?;
        
        let mut current_date = start_date;
        let end_date = start_date + chrono::Duration::days(tournament.duration_days as i64 - 1);
        
        // 開始月から終了月まで、各月に登録
        while current_date <= end_date {
            let year_month = current_date.year() as u32 * 100 + current_date.month();
            let tournament_id = generate_tournament_id(&tournament.venue_name, &tournament.event_name);
            let key = monthly_key(year_month, &tournament_id);
            let value = serialize_to_string(tournament)?;
            self.store.put(key, value)?;
            
            // 次の月に移動
            current_date = if current_date.month() == 12 {
                NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1)
                    .ok_or_else(|| crate::StoreError::InvalidValue)?
            } else {
                NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, 1)
                    .ok_or_else(|| crate::StoreError::InvalidValue)?
            };
            
            // 終了日の月を超えたら終了
            if current_date.year() as u32 * 100 + current_date.month() > 
               end_date.year() as u32 * 100 + end_date.month() {
                break;
            }
        }
        
        Ok(())
    }

    /// データ統計を取得
    /// 
    /// # Returns
    /// (月数, 大会数, レース数) のタプル
    pub fn get_statistics(&mut self) -> Result<(usize, usize, usize)> {
        let all_keys = self.store.keys()?;
        
        let monthly_keys = all_keys.iter().filter(|k| k.starts_with('M')).count();
        let tournament_keys = all_keys.iter().filter(|k| k.starts_with('T')).count();
        
        // 月別ビューの数から大会数を推定
        let unique_tournaments = all_keys
            .iter()
            .filter_map(|k| {
                if k.starts_with('M') {
                    k.split('\x00').nth(1)
                } else {
                    None
                }
            })
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        Ok((monthly_keys, unique_tournaments, tournament_keys))
    }
}

/// 年月文字列をu32に変換 (例: "2025-09" -> 202509)
fn parse_year_month(year_month: &str) -> Result<u32> {
    let parts: Vec<&str> = year_month.split('-').collect();
    if parts.len() != 2 {
        return Err(crate::StoreError::InvalidValue);
    }
    
    let year: u32 = parts[0].parse()
        .map_err(|_| crate::StoreError::InvalidValue)?;
    let month: u32 = parts[1].parse()
        .map_err(|_| crate::StoreError::InvalidValue)?;
    
    if month < 1 || month > 12 {
        return Err(crate::StoreError::InvalidValue);
    }
    
    Ok(year * 100 + month)
}

/// u32を年月文字列に変換 (例: 202509 -> "2025-09")
fn format_year_month(year_month: u32) -> String {
    let year = year_month / 100;
    let month = year_month % 100;
    format!("{:04}-{:02}", year, month)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryStore;

    #[test]
    fn test_parse_year_month() {
        assert_eq!(parse_year_month("2025-09").unwrap(), 202509);
        assert_eq!(parse_year_month("2024-12").unwrap(), 202412);
        assert!(parse_year_month("invalid").is_err());
        assert!(parse_year_month("2025-13").is_err());
    }

    #[test]
    fn test_format_year_month() {
        assert_eq!(format_year_month(202509), "2025-09");
        assert_eq!(format_year_month(202412), "2024-12");
    }

    #[test]
    fn test_put_get_monthly_schedule() {
        let store = MemoryStore::new();
        let mut engine = BoatRaceEngine::new(store);

        let schedule = MonthlySchedule {
            year_month: "2025-09".to_string(),
            events: vec![
                RaceEvent {
                    venue_id: 4,
                    venue_name: "平和島".to_string(),
                    event_name: "トーキョー・ベイ・カップ".to_string(),
                    grade: "G1".to_string(),
                    start_date: "2025-09-10".to_string(),
                    duration_days: 7,
                },
            ],
        };

        // スケジュールを保存
        engine.put_monthly_schedule(&schedule).unwrap();

        // スケジュールを取得
        let retrieved = engine.get_monthly_schedule(202509).unwrap();
        assert_eq!(retrieved.year_month, "2025-09");
        assert_eq!(retrieved.events.len(), 1);
        assert_eq!(retrieved.events[0].venue_name, "平和島");
    }

    #[test]
    fn test_race_data_operations() {
        let store = MemoryStore::new();
        let mut engine = BoatRaceEngine::new(store);

        #[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
        struct RaceData {
            race_number: u32,
            participants: Vec<String>,
        }

        let race_data = RaceData {
            race_number: 1,
            participants: vec!["選手A".to_string(), "選手B".to_string()],
        };

        let tournament_id = "tokyo_bay_cup";
        let timestamp = 1694524800000;

        // レースデータを保存
        engine.put_race_data(tournament_id, timestamp, &race_data).unwrap();

        // レースデータを取得
        let retrieved: RaceData = engine.get_race_data(tournament_id, timestamp).unwrap();
        assert_eq!(retrieved.race_number, 1);
        assert_eq!(retrieved.participants.len(), 2);

        // 大会の全レースを取得
        let all_races: Vec<RaceData> = engine.get_tournament_races(tournament_id).unwrap();
        assert_eq!(all_races.len(), 1);
        assert_eq!(all_races[0], race_data);
    }

    #[test]
    fn test_register_tournament_to_months() {
        let store = MemoryStore::new();
        let mut engine = BoatRaceEngine::new(store);

        let tournament = RaceEvent {
            venue_id: 4,
            venue_name: "平和島".to_string(),
            event_name: "年末年始杯".to_string(),
            grade: "G1".to_string(),
            start_date: "2025-12-28".to_string(),
            duration_days: 10, // 2026-01-06まで
        };

        // 月跨ぎ大会を登録
        engine.register_tournament_to_months(&tournament).unwrap();

        // 12月のスケジュールを確認
        let dec_schedule = engine.get_monthly_schedule(202512).unwrap();
        assert_eq!(dec_schedule.events.len(), 1);
        assert_eq!(dec_schedule.events[0].event_name, "年末年始杯");

        // 1月のスケジュールを確認
        let jan_schedule = engine.get_monthly_schedule(202601).unwrap();
        assert_eq!(jan_schedule.events.len(), 1);
        assert_eq!(jan_schedule.events[0].event_name, "年末年始杯");
    }

    #[test]
    fn test_statistics() {
        let store = MemoryStore::new();
        let mut engine = BoatRaceEngine::new(store);

        let schedule = MonthlySchedule {
            year_month: "2025-09".to_string(),
            events: vec![
                RaceEvent {
                    venue_id: 4,
                    venue_name: "平和島".to_string(),
                    event_name: "トーキョー・ベイ・カップ".to_string(),
                    grade: "G1".to_string(),
                    start_date: "2025-09-10".to_string(),
                    duration_days: 7,
                },
            ],
        };

        engine.put_monthly_schedule(&schedule).unwrap();
        engine.put_race_data("tokyo_bay_cup", 1694524800000, &"race1").unwrap();
        engine.put_race_data("tokyo_bay_cup", 1694524800001, &"race2").unwrap();

        let (monthly_count, tournament_count, race_count) = engine.get_statistics().unwrap();
        assert_eq!(monthly_count, 1); // 1つの月別エントリ
        assert_eq!(tournament_count, 1); // 1つのユニーク大会
        assert_eq!(race_count, 2); // 2つのレース
    }
}