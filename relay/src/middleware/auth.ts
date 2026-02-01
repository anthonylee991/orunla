import type { Context, Next, MiddlewareHandler } from 'hono';
import { db } from '../db/index.js';
import { devices } from '../db/schema.js';
import { eq, and } from 'drizzle-orm';

/**
 * Validates X-Device-ID and X-License-Key headers.
 * Checks that the device exists and belongs to the given license.
 * Sets c.set('deviceId', ...) and c.set('licenseKey', ...) on success.
 */
export function deviceAuth(): MiddlewareHandler {
  return async (c: Context, next: Next) => {
    const deviceId = c.req.header('x-device-id');
    const licenseKey = c.req.header('x-license-key');

    if (!deviceId || !licenseKey) {
      return c.json({ error: 'Missing X-Device-ID or X-License-Key header' }, 401);
    }

    const [device] = await db
      .select({ deviceId: devices.deviceId })
      .from(devices)
      .where(and(eq(devices.deviceId, deviceId), eq(devices.licenseKey, licenseKey)))
      .limit(1);

    if (!device) {
      return c.json({ error: 'Invalid device or license key' }, 401);
    }

    c.set('deviceId', deviceId);
    c.set('licenseKey', licenseKey);

    await next();
  };
}
