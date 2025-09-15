fn sample_data() -> MonthlySchedule {
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
