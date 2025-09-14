# 競艇データ管理システム実装計画

## 概要
- 現在: 単純なHashMap + JSON永続化のKeyValueStore
- 目標: 競艇データ（月別→大会→個別レース）に最適化されたキーバリューストア
- sample.rsの構造を参考にした階層データ管理

## データ構造

### 1. 月別ビュー（軽量メタデータ）
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySchedule {
    pub year_month: String,      // "2025-09"
    pub events: Vec<RaceEvent>,  // 大会一覧
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceEvent {
    pub venue_id: u32,
    pub venue_name: String,
    pub event_name: String,
    pub grade: String,           // "G1", "一般" etc.
    pub start_date: String,      // "2025-09-10"
    pub duration_days: u32,      // 6日間開催など
}
```

### 2. 大会データ本体（詳細データ）
- 個別レース情報（選手データ、オッズ、結果など）
- タイムスタンプ順での時系列データ

## キー設計

### 月別ビューキー
```
キー: M + <YYYYMM> + 0x00 + <tournament_id>
値: RaceEvent構造体（bincodeでシリアライズ）

例:
M + 202509 + 0x00 + "tokyo_bay_cup" → RaceEvent { venue_id: 4, event_name: "トーキョー・ベイ・カップ", ... }
M + 202509 + 0x00 + "takamatsu_cup" → RaceEvent { venue_id: 12, event_name: "高松宮記念", ... }
```

### 大会データ本体キー
```
キー: T + <tournament_id> + 0x00 + <timestamp_be>
値: 個別レース構造体（直接シリアライズ）

例:
T + "tokyo_bay_cup" + 0x00 + <1R_timestamp> → Race構造体（1Rの詳細データ）
T + "tokyo_bay_cup" + 0x00 + <2R_timestamp> → Race構造体（2Rの詳細データ）
...
T + "tokyo_bay_cup" + 0x00 + <12R_timestamp> → Race構造体（12Rの詳細データ）
```

## 検索パターン

### 1. 月別大会一覧表示
```rust
// M+202509 をスキャンして大会一覧を取得
let key_prefix = format!("M{:06}", 202509);
let events = store.scan(&key_prefix, &next_month_prefix);
```

### 2. 大会内レース一覧表示
```rust
// T+tournament_id をスキャンして全レース取得
let key_prefix = format!("T{}\x00", tournament_id);
let races = store.scan(&key_prefix, &next_tournament_prefix);
```

### 3. 特定レースの詳細表示
```rust
// T+tournament_id+timestamp で個別レース取得
let key = format!("T{}\x00{}", tournament_id, timestamp_be);
let race_detail = store.get(&key);
```

## Phase 1: 基盤整備

### Step 1: 依存関係とモジュール構造
**作業内容**:
- Cargo.tomlに追加:
  ```toml
  chrono = { version = "0.4", features = ["serde", "clock"] }
  thiserror = "1"
  bincode = "1"
  ```
- 新しいファイル構造:
  ```
  src/
  ├── lib.rs          // 既存（モジュール公開追加）
  ├── error.rs        // 既存
  ├── store.rs        // 既存（scan機能追加）
  ├── key.rs          // 新規：競艇データキー管理
  ├── value.rs        // 新規：JSON処理
  └── engine.rs       // 新規：競艇データエンジン
  ```

### Step 2: KeyValueStoreトレイト拡張
**作業内容**:
- store.rsに`scan(start_key, end_key) -> Vec<(String, String)>`メソッド追加
- MemoryStore, FileStore両方に実装

## Phase 2: コア機能実装

### Step 3: 競艇データキー管理（src/key.rs）
**実装内容**:
- キープレフィックス定義:
  ```rust
  pub const PREFIX_MONTHLY: u8 = b'M';     // 月別ビュー
  pub const PREFIX_TOURNAMENT: u8 = b'T';  // 大会データ
  pub const SEPARATOR: u8 = 0x00;          // セパレータ
  ```
- キー生成関数:
  ```rust
  pub fn monthly_key(year_month: u32, tournament_id: &str) -> String
  pub fn tournament_key(tournament_id: &str, timestamp: u64) -> String
  pub fn monthly_scan_range(year_month: u32) -> (String, String)
  pub fn tournament_scan_range(tournament_id: &str) -> (String, String)
  ```

### Step 4: 構造体値処理（src/value.rs）
**実装内容**:
- sample.rsのデータ構造をベースにしたバイナリシリアライズ処理
- `MonthlySchedule`, `RaceEvent`のbincodeシリアライズ/デシリアライズ
- 型安全な個別レースデータ処理（ジェネリクス対応）

### Step 5: 競艇データエンジン（src/engine.rs）
**実装内容**:
```rust
pub struct BoatRaceEngine<K: KeyValueStore> {
    store: K,
}

impl<K: KeyValueStore> BoatRaceEngine<K> {
    // 月別ビュー管理
    pub fn put_monthly_schedule(&mut self, schedule: &MonthlySchedule) -> Result<()>
    pub fn get_monthly_schedule(&self, year_month: u32) -> Result<MonthlySchedule>
    
    // 大会データ管理
    pub fn put_race_data<T: Serialize>(&mut self, tournament_id: &str, timestamp: u64, data: &T) -> Result<()>
    pub fn get_tournament_races<T: DeserializeOwned>(&self, tournament_id: &str) -> Result<Vec<T>>
    pub fn get_race_data<T: DeserializeOwned>(&self, tournament_id: &str, timestamp: u64) -> Result<T>
    
    // 月跨ぎ大会対応
    pub fn register_tournament_to_months(&mut self, tournament: &RaceEvent) -> Result<()>
}
```

## Phase 3: 検証とデモ

### Step 6: テストとデモ作成
**作業内容**:
- examples/boat_race_demo.rs作成
- sample.rsのデータを使用したデモンストレーション
- 月表示→大会選択→レース詳細の流れを実装

## 技術的な特徴

### 高速な月別表示
- 月別ビューは軽量メタデータのみ
- 大量の個別レースデータを読み込まずに大会一覧表示

### 効率的な大会データ管理
- 大会IDでグループ化された時系列データ
- 単一スキャンで大会内全レース取得

### 月跨ぎ大会対応
- 9月開始→10月終了の大会は両方の月に登録
- 各月のビューから同じ大会にアクセス可能

### スケーラビリティ
- データ量に比例しない月別表示性能
- 大会数に比例する軽量な月別インデックス