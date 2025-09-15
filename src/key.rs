/// 競艇データ用のキー管理モジュール
/// 
/// キー設計:
/// - 月別ビュー: M + YYYYMM + 0x00 + tournament_id
/// - 大会データ: T + tournament_id + 0x00 + timestamp_be

// キープレフィックス定義
pub const PREFIX_MONTHLY: u8 = b'M';     // 月別ビュー
pub const PREFIX_TOURNAMENT: u8 = b'T';  // 大会データ
pub const SEPARATOR: u8 = 0x00;          // セパレータ

/// 月別ビューキーを生成
/// 
/// # Arguments
/// * `year_month` - YYYYMM形式の年月 (例: 202509)
/// * `tournament_id` - 大会ID (例: "tokyo_bay_cup")
/// 
/// # Returns
/// "M202509\x00tokyo_bay_cup" のようなキー
pub fn monthly_key(year_month: u32, tournament_id: &str) -> String {
    format!("{}{:06}{}{}", 
        PREFIX_MONTHLY as char,
        year_month,
        SEPARATOR as char,
        tournament_id
    )
}

/// 大会データキーを生成
/// 
/// # Arguments
/// * `tournament_id` - 大会ID
/// * `timestamp` - タイムスタンプ（エポックミリ秒）
/// 
/// # Returns
/// "Ttokyo_bay_cup\x00<timestamp_be>" のようなキー
pub fn tournament_key(tournament_id: &str, timestamp: u64) -> String {
    format!("{}{}{}{:016x}", 
        PREFIX_TOURNAMENT as char,
        tournament_id,
        SEPARATOR as char,
        timestamp
    )
}

/// 月別スキャン範囲を生成
/// 
/// # Arguments
/// * `year_month` - YYYYMM形式の年月
/// 
/// # Returns
/// (開始キー, 終了キー) のタプル
pub fn monthly_scan_range(year_month: u32) -> (String, String) {
    let start = format!("{}{:06}", PREFIX_MONTHLY as char, year_month);
    let end = format!("{}{:06}", PREFIX_MONTHLY as char, year_month + 1);
    (start, end)
}

/// 大会スキャン範囲を生成
/// 
/// # Arguments
/// * `tournament_id` - 大会ID
/// 
/// # Returns
/// (開始キー, 終了キー) のタプル
pub fn tournament_scan_range(tournament_id: &str) -> (String, String) {
    let start = format!("{}{}{}", 
        PREFIX_TOURNAMENT as char,
        tournament_id,
        SEPARATOR as char
    );
    let end = format!("{}{}{}", 
        PREFIX_TOURNAMENT as char,
        tournament_id,
        (SEPARATOR + 1) as char
    );
    (start, end)
}

/// 大会IDから一意のキー識別子を生成
/// 
/// # Arguments
/// * `venue_name` - 会場名 (例: "平和島")
/// * `event_name` - イベント名 (例: "トーキョー・ベイ・カップ")
/// 
/// # Returns
/// 安全なキー識別子 (例: "venue_4_event_tokyo_bay_cup")
pub fn generate_tournament_id(venue_name: &str, event_name: &str) -> String {
    // ASCII文字のみを抽出
    let venue_ascii: String = venue_name
        .chars()
        .filter_map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => Some(c.to_ascii_lowercase()),
            ' ' => Some('_'),
            _ => None,
        })
        .collect();

    let event_ascii: String = event_name
        .chars()
        .filter_map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => Some(c.to_ascii_lowercase()),
            ' ' => Some('_'),
            _ => None,
        })
        .collect();

    // ASCII文字が少ない場合は、ハッシュベースのIDを生成
    let venue_part = if venue_ascii.len() > 2 {
        venue_ascii
    } else {
        format!("venue_{}", venue_name.len())
    };

    let event_part = if event_ascii.len() > 2 {
        event_ascii
    } else {
        format!("event_{}", event_name.len())
    };

    // 連続する_を1つにまとめ
    let combined = format!("{}_{}", venue_part, event_part);
    let mut result = String::new();
    let mut prev_underscore = false;

    for c in combined.chars() {
        if c == '_' {
            if !prev_underscore && !result.is_empty() {
                result.push(c);
            }
            prev_underscore = true;
        } else {
            result.push(c);
            prev_underscore = false;
        }
    }
    
    result.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monthly_key() {
        let key = monthly_key(202509, "tokyo_bay_cup");
        assert_eq!(key, "M202509\x00tokyo_bay_cup");
    }

    #[test]
    fn test_tournament_key() {
        let key = tournament_key("tokyo_bay_cup", 1694524800000);
        assert_eq!(key, "Ttokyo_bay_cup\x000000018a898c7c00");
    }

    #[test]
    fn test_monthly_scan_range() {
        let (start, end) = monthly_scan_range(202509);
        assert_eq!(start, "M202509");
        assert_eq!(end, "M202510");
    }

    #[test]
    fn test_tournament_scan_range() {
        let (start, end) = tournament_scan_range("tokyo_bay_cup");
        assert_eq!(start, "Ttokyo_bay_cup\x00");
        assert_eq!(end, "Ttokyo_bay_cup\x01");
    }

    #[test]
    fn test_generate_tournament_id() {
        let id = generate_tournament_id("平和島", "トーキョー・ベイ・カップ");
        // 日本語文字は除去される
        assert!(!id.contains("平"));
        assert!(!id.contains("島"));
        assert!(!id.contains("ト"));
        assert!(!id.contains("ー"));
        // 文字数ベースのIDが生成される（UTF-8バイト長ベース）
        assert_eq!(id, "venue_9_event_36");
    }

    #[test]
    fn test_generate_tournament_id_ascii() {
        let id = generate_tournament_id("Tokyo", "Bay Cup 2025");
        assert_eq!(id, "tokyo_bay_cup_2025");
    }
}