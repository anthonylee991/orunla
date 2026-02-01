import {
  pgTable,
  text,
  uuid,
  timestamp,
  bigserial,
  bigint,
  varchar,
  index,
} from 'drizzle-orm/pg-core';

/**
 * Registered devices. Each desktop app instance registers here on first sync.
 * device_id is client-generated (UUID stored in local SQLite sync_state table).
 * license_key links to Supabase purchases.token (validated server-side).
 */
export const devices = pgTable(
  'devices',
  {
    deviceId: varchar('device_id', { length: 255 }).primaryKey(),
    licenseKey: uuid('license_key').notNull(),
    deviceName: varchar('device_name', { length: 255 }),
    lastSeen: timestamp('last_seen', { withTimezone: true }).defaultNow().notNull(),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    licenseIdx: index('idx_devices_license').on(table.licenseKey),
  }),
);

/**
 * Sync event buffer. Stores encrypted changelog events from all devices.
 * Each event is an encrypted JSON blob -- relay never sees plaintext.
 * Pruned after 30 days.
 */
export const syncEvents = pgTable(
  'sync_events',
  {
    id: bigserial('id', { mode: 'number' }).primaryKey(),
    deviceId: varchar('device_id', { length: 255 })
      .notNull()
      .references(() => devices.deviceId, { onDelete: 'cascade' }),
    payload: text('payload').notNull(),
    vectorClock: bigint('vector_clock', { mode: 'number' }).notNull(),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  },
  (table) => ({
    deviceClockIdx: index('idx_sync_events_device_clock').on(table.deviceId, table.vectorClock),
  }),
);
