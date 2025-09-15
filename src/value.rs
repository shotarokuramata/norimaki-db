/// 構造体値処理モジュール
/// 
/// bincodeを使用した型安全なシリアライズ/デシリアライズ機能を提供

use crate::{Result, StoreError};
use serde::{Deserialize, Serialize};

/// 任意の構造体をバイナリ形式でシリアライズ
/// 
/// # Arguments
/// * `value` - シリアライズする構造体
/// 
/// # Returns
/// バイナリデータ
pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value).map_err(|e| StoreError::SerializationError(format!("Serialize error: {}", e)))
}

/// バイナリデータから構造体にデシリアライズ
/// 
/// # Arguments
/// * `data` - バイナリデータ
/// 
/// # Returns
/// デシリアライズされた構造体
pub fn deserialize<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T> {
    bincode::deserialize(data).map_err(|e| StoreError::SerializationError(format!("Deserialize error: {}", e)))
}

/// 構造体をKeyValueStoreに格納するためのString形式に変換
/// 
/// # Arguments
/// * `value` - シリアライズする構造体
/// 
/// # Returns
/// Base64エンコードされた文字列
pub fn serialize_to_string<T: Serialize>(value: &T) -> Result<String> {
    use base64::{Engine as _, engine::general_purpose};
    let binary = serialize(value)?;
    Ok(general_purpose::STANDARD.encode(binary))
}

/// String形式から構造体にデシリアライズ
/// 
/// # Arguments
/// * `data` - Base64エンコードされた文字列
/// 
/// # Returns
/// デシリアライズされた構造体
pub fn deserialize_from_string<T: for<'de> Deserialize<'de>>(data: &str) -> Result<T> {
    use base64::{Engine as _, engine::general_purpose};
    let binary = general_purpose::STANDARD.decode(data)
        .map_err(|e| StoreError::SerializationError(format!("Base64 decode error: {}", e)))?;
    deserialize(&binary)
}

/// 構造体の大きさを効率的に計算
/// 
/// # Arguments
/// * `value` - 計算対象の構造体
/// 
/// # Returns
/// シリアライズ後のバイト数
pub fn calculate_size<T: Serialize>(value: &T) -> Result<usize> {
    let binary = serialize(value)?;
    Ok(binary.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MonthlySchedule, RaceEvent};

    #[test]
    fn test_serialize_deserialize() {
        let event = RaceEvent {
            venue_id: 4,
            venue_name: "平和島".to_string(),
            event_name: "トーキョー・ベイ・カップ".to_string(),
            grade: "G1".to_string(),
            start_date: "2025-09-10".to_string(),
            duration_days: 7,
        };

        // シリアライズ
        let binary = serialize(&event).unwrap();
        assert!(!binary.is_empty());

        // デシリアライズ
        let restored: RaceEvent = deserialize(&binary).unwrap();
        assert_eq!(restored.venue_id, event.venue_id);
        assert_eq!(restored.venue_name, event.venue_name);
        assert_eq!(restored.event_name, event.event_name);
        assert_eq!(restored.grade, event.grade);
        assert_eq!(restored.start_date, event.start_date);
        assert_eq!(restored.duration_days, event.duration_days);
    }

    #[test]
    fn test_serialize_to_string() {
        let event = RaceEvent {
            venue_id: 1,
            venue_name: "桐生".to_string(),
            event_name: "群馬クレインサンダーズカップ".to_string(),
            grade: "一般".to_string(),
            start_date: "2025-09-11".to_string(),
            duration_days: 6,
        };

        // String形式でシリアライズ
        let encoded = serialize_to_string(&event).unwrap();
        assert!(!encoded.is_empty());
        
        // String形式からデシリアライズ
        let restored: RaceEvent = deserialize_from_string(&encoded).unwrap();
        assert_eq!(restored.venue_name, "桐生");
        assert_eq!(restored.grade, "一般");
    }

    #[test]
    fn test_monthly_schedule_serialization() {
        // テストデータを直接作成
        let schedule = MonthlySchedule {
            year_month: "2025-09".to_string(),
            events: vec![
                RaceEvent {
                    venue_id: 1,
                    venue_name: "桐生".to_string(),
                    event_name: "群馬クレインサンダーズカップ".to_string(),
                    grade: "一般".to_string(),
                    start_date: "2025-09-11".to_string(),
                    duration_days: 6,
                },
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

        // MonthlyScheduleのシリアライズテスト
        let encoded = serialize_to_string(&schedule).unwrap();
        let restored: MonthlySchedule = deserialize_from_string(&encoded).unwrap();

        assert_eq!(restored.year_month, schedule.year_month);
        assert_eq!(restored.events.len(), schedule.events.len());
        assert_eq!(restored.events[0].venue_name, schedule.events[0].venue_name);
        assert_eq!(restored.events[1].grade, "G1");
    }

    #[test]
    fn test_calculate_size() {
        let event = RaceEvent {
            venue_id: 4,
            venue_name: "平和島".to_string(),
            event_name: "トーキョー・ベイ・カップ".to_string(),
            grade: "G1".to_string(),
            start_date: "2025-09-10".to_string(),
            duration_days: 7,
        };

        let size = calculate_size(&event).unwrap();
        assert!(size > 0);
        
        // 実際のシリアライズ結果と同じサイズであることを確認
        let binary = serialize(&event).unwrap();
        assert_eq!(size, binary.len());
    }

    #[test]
    fn test_invalid_data_deserialization() {
        // 無効なBase64データでのデシリアライズテスト
        let result: Result<RaceEvent> = deserialize_from_string("invalid_base64!");
        assert!(result.is_err());

        // 無効なバイナリデータでのデシリアライズテスト
        let invalid_binary = vec![0xFF, 0xFE, 0xFD];
        let result: Result<RaceEvent> = deserialize(&invalid_binary);
        assert!(result.is_err());
    }
}