-- Orunla Relay: initial schema
-- Devices registered for cross-device sync
CREATE TABLE IF NOT EXISTS devices (
    device_id VARCHAR(255) PRIMARY KEY,
    license_key UUID NOT NULL,
    device_name VARCHAR(255),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_devices_license ON devices(license_key);

-- Encrypted sync event buffer (pruned after 30 days)
CREATE TABLE IF NOT EXISTS sync_events (
    id BIGSERIAL PRIMARY KEY,
    device_id VARCHAR(255) NOT NULL REFERENCES devices(device_id) ON DELETE CASCADE,
    payload TEXT NOT NULL,
    vector_clock BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_sync_events_device_clock ON sync_events(device_id, vector_clock);
