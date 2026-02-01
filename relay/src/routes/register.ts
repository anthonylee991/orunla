import { Hono } from 'hono';
import { rateLimiter } from '../middleware/rate-limit.js';
import { db } from '../db/index.js';
import { devices } from '../db/schema.js';
import { eq } from 'drizzle-orm';

const SUPABASE_URL = process.env.SUPABASE_URL;
const SUPABASE_SERVICE_KEY = process.env.SUPABASE_SERVICE_KEY;
const PRODUCT_ID = 'orunla_standard';

export const registerRoutes = new Hono();

/**
 * POST /orunla/register
 * Headers: X-License-Key
 * Body: { device_id: string, device_name?: string }
 * Response: { status: 'ok', device_id: string }
 *
 * Registers a new device for sync. Validates the license key against Supabase
 * server-side before allowing registration.
 */
registerRoutes.post('/', rateLimiter(10), async (c) => {
  const licenseKey = c.req.header('x-license-key');
  if (!licenseKey) {
    return c.json({ error: 'Missing X-License-Key header' }, 401);
  }

  const body = await c.req.json<{ device_id?: string; device_name?: string }>().catch(() => ({}));
  const deviceId = (body as { device_id?: string }).device_id?.trim();
  const deviceName = (body as { device_name?: string }).device_name?.trim() || null;

  if (!deviceId || deviceId.length === 0 || deviceId.length > 255) {
    return c.json({ error: 'Invalid device_id' }, 400);
  }

  // Validate license key against Supabase
  if (!SUPABASE_URL || !SUPABASE_SERVICE_KEY) {
    return c.json({ error: 'License service unavailable' }, 503);
  }

  try {
    const res = await fetch(
      `${SUPABASE_URL}/rest/v1/purchases?token=eq.${encodeURIComponent(licenseKey)}&product_id=eq.${PRODUCT_ID}&select=id`,
      {
        headers: {
          apikey: SUPABASE_SERVICE_KEY,
          Authorization: `Bearer ${SUPABASE_SERVICE_KEY}`,
        },
      },
    );

    if (!res.ok) {
      return c.json({ error: 'License service unavailable' }, 503);
    }

    const rows = await res.json();
    if (!Array.isArray(rows) || rows.length === 0) {
      return c.json({ error: 'Invalid license key' }, 401);
    }
  } catch {
    return c.json({ error: 'License service unavailable' }, 503);
  }

  // Upsert device
  const existing = await db
    .select({ deviceId: devices.deviceId })
    .from(devices)
    .where(eq(devices.deviceId, deviceId))
    .limit(1);

  if (existing.length > 0) {
    // Update last_seen and device_name
    await db
      .update(devices)
      .set({ lastSeen: new Date(), deviceName })
      .where(eq(devices.deviceId, deviceId));
  } else {
    await db.insert(devices).values({
      deviceId,
      licenseKey,
      deviceName,
    });
  }

  return c.json({ status: 'ok', device_id: deviceId });
});
