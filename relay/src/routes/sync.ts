import { Hono } from 'hono';
import { deviceAuth } from '../middleware/auth.js';
import { rateLimiter } from '../middleware/rate-limit.js';
import { db } from '../db/index.js';
import { devices, syncEvents } from '../db/schema.js';
import { eq, and, gt, ne, asc } from 'drizzle-orm';
import type { AppEnv } from '../types.js';

export const syncRoutes = new Hono<AppEnv>();

// All sync routes require device auth
syncRoutes.use('/*', deviceAuth());
syncRoutes.use('/*', rateLimiter(120));

/**
 * POST /orunla/push
 * Headers: X-Device-ID, X-License-Key
 * Body: { events: [{ id, payload, vector_clock, created_at }] }
 * Response: { status: 'ok', accepted_count: number }
 *
 * Upload encrypted changelog events from this device.
 */
syncRoutes.post('/push', async (c) => {
  const deviceId = c.get('deviceId');

  const body = await c.req.json<{
    events?: Array<{
      id: string;
      payload: string;
      vector_clock: number;
      created_at: string;
    }>;
  }>();

  const events = body?.events;
  if (!events || !Array.isArray(events) || events.length === 0) {
    return c.json({ status: 'ok', accepted_count: 0 });
  }

  // Limit batch size
  if (events.length > 500) {
    return c.json({ error: 'Batch too large (max 500 events)' }, 400);
  }

  // Bulk insert
  const rows = events.map((e) => ({
    deviceId,
    payload: e.payload,
    vectorClock: e.vector_clock,
    createdAt: new Date(e.created_at),
  }));

  await db.insert(syncEvents).values(rows);

  // Update device last_seen
  await db
    .update(devices)
    .set({ lastSeen: new Date() })
    .where(eq(devices.deviceId, deviceId));

  return c.json({ status: 'ok', accepted_count: events.length });
});

/**
 * GET /orunla/pull?since=<vector_clock>&limit=500
 * Headers: X-Device-ID, X-License-Key
 * Response: { events: [{ id, device_id, payload, vector_clock, created_at }], latest_clock: number }
 *
 * Download events from OTHER devices since the given clock value.
 * Excludes events from the requesting device (they already have those).
 */
syncRoutes.get('/pull', async (c) => {
  const deviceId = c.get('deviceId');
  const licenseKey = c.get('licenseKey');
  const since = parseInt(c.req.query('since') || '0', 10);
  const limit = Math.min(parseInt(c.req.query('limit') || '500', 10), 500);

  // Find all devices on the same license (for cross-device sync)
  const sameAccountDevices = await db
    .select({ deviceId: devices.deviceId })
    .from(devices)
    .where(eq(devices.licenseKey, licenseKey));

  const otherDeviceIds = sameAccountDevices
    .map((d) => d.deviceId)
    .filter((id) => id !== deviceId);

  if (otherDeviceIds.length === 0) {
    return c.json({ events: [], latest_clock: since });
  }

  // Pull events from other devices with the same license, since the given clock
  const events = await db
    .select({
      id: syncEvents.id,
      deviceId: syncEvents.deviceId,
      payload: syncEvents.payload,
      vectorClock: syncEvents.vectorClock,
      createdAt: syncEvents.createdAt,
    })
    .from(syncEvents)
    .where(
      and(
        gt(syncEvents.id, since),
        ne(syncEvents.deviceId, deviceId),
        // Only events from devices on the same license
        ...(otherDeviceIds.length > 0
          ? [
              // Using SQL IN via a filter approach
            ]
          : []),
      ),
    )
    .orderBy(asc(syncEvents.id))
    .limit(limit);

  // Filter to same-license devices (post-query filter for simplicity)
  const filtered = events.filter((e) => otherDeviceIds.includes(e.deviceId));

  const latestClock =
    filtered.length > 0 ? Math.max(...filtered.map((e) => e.id)) : since;

  return c.json({
    events: filtered.map((e) => ({
      id: e.id,
      device_id: e.deviceId,
      payload: e.payload,
      vector_clock: e.vectorClock,
      created_at: e.createdAt?.toISOString(),
    })),
    latest_clock: latestClock,
  });
});

/**
 * POST /orunla/heartbeat
 * Headers: X-Device-ID, X-License-Key
 * Response: { status: 'ok', sync_enabled: true }
 *
 * Updates last_seen timestamp. Used to keep device registration fresh
 * and verify license is still valid.
 */
syncRoutes.post('/heartbeat', async (c) => {
  const deviceId = c.get('deviceId');

  await db
    .update(devices)
    .set({ lastSeen: new Date() })
    .where(eq(devices.deviceId, deviceId));

  return c.json({ status: 'ok', sync_enabled: true });
});
