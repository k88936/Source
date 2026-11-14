//! The settings the app stores, and the clocks the schedule reads.
//!
//! Every value is one key in the catalog's `state` table, so writing one
//! setting never touches the others. A delta says which ones a page changed and
//! is applied as it is: a missing field is not "reset to default", it is a
//! setting this screen does not own.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::store::{CatalogError, SettingsStore};

/// Whether the platform should hold the process in the background.
pub const KEEP_ALIVE_KEY: &str = "keep.alive";
/// How often every local album is walked again.
pub const SYNC_FREQUENCY_KEY: &str = "sync.frequency";
/// How often every cloud album's listing is read again.
pub const INDEX_FREQUENCY_KEY: &str = "index.frequency";
/// When the local walk last ran on its own.
pub const SYNC_RAN_KEY: &str = "sync.lastAutoRun";
/// When the cloud listing pass last ran on its own.
pub const INDEX_RAN_KEY: &str = "index.lastAutoRun";
/// How often a service looks at its own schedule.
pub const SCHEDULE_TICK: Duration = Duration::from_secs(60);

/// How often a whole set is walked again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Frequency {
    Never,
    Daily,
    Weekly,
    Monthly,
}

impl Frequency {
    fn as_str(self) -> &'static str {
        match self {
            Frequency::Never => "never",
            Frequency::Daily => "daily",
            Frequency::Weekly => "weekly",
            Frequency::Monthly => "monthly",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "never" => Some(Frequency::Never),
            "daily" => Some(Frequency::Daily),
            "weekly" => Some(Frequency::Weekly),
            "monthly" => Some(Frequency::Monthly),
            _ => None,
        }
    }

    /// How long a run stays fresh.
    fn interval_ms(self) -> i64 {
        match self {
            Frequency::Never => i64::MAX,
            Frequency::Daily => 24 * 60 * 60 * 1000,
            Frequency::Weekly => 7 * 24 * 60 * 60 * 1000,
            Frequency::Monthly => 30 * 24 * 60 * 60 * 1000,
        }
    }
}

/// What the settings page reads.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Settings {
    pub keep_alive: bool,
    pub sync_frequency: Frequency,
    pub index_frequency: Frequency,
}

/// Which settings a page changed.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SettingsDelta {
    #[ts(optional)]
    pub keep_alive: Option<bool>,
    #[ts(optional)]
    pub sync_frequency: Option<Frequency>,
    #[ts(optional)]
    pub index_frequency: Option<Frequency>,
}

/// The stored settings.
pub struct SettingsService {
    settings: Arc<dyn SettingsStore>,
}

impl SettingsService {
    pub fn new(settings: Arc<dyn SettingsStore>) -> Self {
        Self { settings }
    }

    pub async fn get(&self) -> Result<Settings, CatalogError> {
        Ok(Settings {
            keep_alive: self.settings.get_setting(KEEP_ALIVE_KEY).await?.as_deref() != Some("0"),
            sync_frequency: self.frequency(SYNC_FREQUENCY_KEY).await?,
            index_frequency: self.frequency(INDEX_FREQUENCY_KEY).await?,
        })
    }

    pub async fn set(&self, delta: &SettingsDelta) -> Result<(), CatalogError> {
        if let Some(keep_alive) = delta.keep_alive {
            let value = if keep_alive { "1" } else { "0" };
            self.settings.set_setting(KEEP_ALIVE_KEY, value).await?;
        }
        if let Some(frequency) = delta.sync_frequency {
            self.settings
                .set_setting(SYNC_FREQUENCY_KEY, frequency.as_str())
                .await?;
        }
        if let Some(frequency) = delta.index_frequency {
            self.settings
                .set_setting(INDEX_FREQUENCY_KEY, frequency.as_str())
                .await?;
        }
        Ok(())
    }

    async fn frequency(&self, key: &str) -> Result<Frequency, CatalogError> {
        Ok(stored_frequency(&self.settings.get_setting(key).await?))
    }
}

/// When a periodic walk is due, and when it last ran.
///
/// The clock is kept apart from the frequency: only a run the schedule started
/// moves it, so scanning a directory by hand does not push the next automatic
/// one back.
pub struct Schedule {
    settings: Arc<dyn SettingsStore>,
}

impl Schedule {
    pub fn new(settings: Arc<dyn SettingsStore>) -> Self {
        Self { settings }
    }

    /// Whether `frequency` is due now, given when it last ran on its own.
    ///
    /// A frequency that was never run is due: turning the setting on asks for a
    /// pass, not for a wait.
    pub async fn due(&self, frequency: Frequency, ran_key: &str) -> Result<bool, CatalogError> {
        if frequency == Frequency::Never {
            return Ok(false);
        }
        let last = self
            .settings
            .get_setting(ran_key)
            .await?
            .and_then(|value| value.parse::<i64>().ok());
        Ok(match last {
            Some(last) => now_ms() - last >= frequency.interval_ms(),
            None => true,
        })
    }

    /// Record that a pass was started, so the next one waits a full interval.
    pub async fn mark_ran(&self, ran_key: &str) -> Result<(), CatalogError> {
        self.settings
            .set_setting(ran_key, &now_ms().to_string())
            .await
    }

    /// Whether the local walk is due.
    pub async fn sync_due(&self) -> Result<bool, CatalogError> {
        let frequency = stored_frequency(&self.settings.get_setting(SYNC_FREQUENCY_KEY).await?);
        self.due(frequency, SYNC_RAN_KEY).await
    }

    /// Record that a local walk was started.
    pub async fn mark_sync_ran(&self) -> Result<(), CatalogError> {
        self.mark_ran(SYNC_RAN_KEY).await
    }

    /// Whether the cloud listing pass is due.
    pub async fn index_due(&self) -> Result<bool, CatalogError> {
        let frequency = stored_frequency(&self.settings.get_setting(INDEX_FREQUENCY_KEY).await?);
        self.due(frequency, INDEX_RAN_KEY).await
    }

    /// Record that a cloud listing pass was started.
    pub async fn mark_index_ran(&self) -> Result<(), CatalogError> {
        self.mark_ran(INDEX_RAN_KEY).await
    }
}

/// The stored frequency, or `Never` when nothing was chosen.
fn stored_frequency(stored: &Option<String>) -> Frequency {
    stored
        .as_deref()
        .and_then(Frequency::parse)
        .unwrap_or(Frequency::Never)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as i64)
        .unwrap_or(0)
}
