use crate::licensing::store::{License, LicenseStore};
use crate::licensing::tier::Tier;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::Deserialize;

/// Relay URL -- the only external endpoint the client needs to know.
/// Supabase credentials live server-side on Railway, never in the binary.
const RELAY_URL: &str = "https://orunla-production.up.railway.app";

const TRIAL_DAYS: i64 = 14;
const REVALIDATION_DAYS: i64 = 7;
const GRACE_PERIOD_DAYS: i64 = 3;

#[derive(Deserialize)]
struct ValidateResponse {
    valid: bool,
}

pub struct LicenseValidator {
    client: reqwest::Client,
}

impl LicenseValidator {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Validate a license key via the relay API.
    /// Relay checks Supabase server-side; client never touches Supabase directly.
    /// Returns Tier::Pro if valid, error if invalid.
    pub async fn validate_token(&self, license_key: &str) -> Result<Tier> {
        let response = self
            .client
            .post(format!("{}/v1/license/validate", RELAY_URL))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "key": license_key }))
            .send()
            .await
            .context("Could not reach license server")?;

        if !response.status().is_success() {
            anyhow::bail!("License server returned status {}", response.status());
        }

        let data: ValidateResponse = response
            .json()
            .await
            .context("Invalid response from license server")?;

        if data.valid {
            Ok(Tier::Pro)
        } else {
            anyhow::bail!("Invalid license key")
        }
    }

    /// Activate a license key: validate via relay and store locally.
    pub async fn activate(
        &self,
        license_key: &str,
        store: &LicenseStore,
    ) -> Result<Tier> {
        let tier = self.validate_token(license_key).await?;

        let license = License {
            license_key: license_key.to_string(),
            tier,
            trial_start: None,
            last_validated: Utc::now(),
        };
        store.save(&license)?;

        Ok(tier)
    }

    /// Check the current tier, handling trial expiry and revalidation.
    /// Updates the stored license if tier changes.
    pub async fn get_tier(&self, store: &LicenseStore) -> Result<Tier> {
        let mut license = store.ensure_license()?;
        let now = Utc::now();

        match license.tier {
            Tier::Trial => {
                if let Some(trial_start) = license.trial_start {
                    let elapsed = now.signed_duration_since(trial_start).num_days();
                    if elapsed > TRIAL_DAYS {
                        license.tier = Tier::Free;
                        store.save(&license)?;
                        return Ok(Tier::Free);
                    }
                }
                Ok(Tier::Trial)
            }
            Tier::Pro => {
                let days_since = now
                    .signed_duration_since(license.last_validated)
                    .num_days();

                if days_since > REVALIDATION_DAYS + GRACE_PERIOD_DAYS {
                    license.tier = Tier::Free;
                    store.save(&license)?;
                    return Ok(Tier::Free);
                }

                if days_since > REVALIDATION_DAYS {
                    match self.validate_token(&license.license_key).await {
                        Ok(tier) => {
                            license.tier = tier;
                            license.last_validated = now;
                            store.save(&license)?;
                            Ok(tier)
                        }
                        Err(_) => {
                            // Still within grace period, keep Pro
                            Ok(Tier::Pro)
                        }
                    }
                } else {
                    Ok(Tier::Pro)
                }
            }
            Tier::Free => Ok(Tier::Free),
        }
    }

    /// Get the current tier synchronously (local state only, no network).
    pub fn get_tier_local(store: &LicenseStore) -> Result<Tier> {
        let license = store.ensure_license()?;
        let now = Utc::now();

        match license.tier {
            Tier::Trial => {
                if let Some(trial_start) = license.trial_start {
                    let elapsed = now.signed_duration_since(trial_start).num_days();
                    if elapsed > TRIAL_DAYS {
                        return Ok(Tier::Free);
                    }
                }
                Ok(Tier::Trial)
            }
            Tier::Pro => {
                let days_since = now
                    .signed_duration_since(license.last_validated)
                    .num_days();
                if days_since > REVALIDATION_DAYS + GRACE_PERIOD_DAYS {
                    Ok(Tier::Free)
                } else {
                    Ok(Tier::Pro)
                }
            }
            Tier::Free => Ok(Tier::Free),
        }
    }

    /// Get trial days remaining (None if not on trial)
    pub fn trial_days_remaining(store: &LicenseStore) -> Result<Option<i64>> {
        let license = store.ensure_license()?;
        if license.tier != Tier::Trial {
            return Ok(None);
        }
        if let Some(trial_start) = license.trial_start {
            let elapsed = Utc::now().signed_duration_since(trial_start).num_days();
            Ok(Some((TRIAL_DAYS - elapsed).max(0)))
        } else {
            Ok(None)
        }
    }
}
