import { Hono } from 'hono';
import { rateLimiter } from '../middleware/rate-limit.js';

const SUPABASE_URL = process.env.SUPABASE_URL;
const SUPABASE_SERVICE_KEY = process.env.SUPABASE_SERVICE_KEY;
const PRODUCT_ID = 'orunla_standard';

export const licenseRoutes = new Hono();

/**
 * POST /v1/license/validate
 * Body: { key: string }
 * Response: { valid: boolean }
 *
 * Server-side relay: queries Supabase purchases table using service role key.
 * Desktop app never touches Supabase directly.
 * Rate limited at 5 req/min per IP to prevent brute force.
 */
licenseRoutes.post('/validate', rateLimiter(5), async (c) => {
  if (!SUPABASE_URL || !SUPABASE_SERVICE_KEY) {
    return c.json({ error: 'License service unavailable' }, 503);
  }

  const body = await c.req.json<{ key?: string }>().catch(() => ({}));
  const key = (body as { key?: string }).key?.trim();

  if (!key || key.length === 0 || key.length > 500) {
    return c.json({ valid: false }, 200);
  }

  try {
    const res = await fetch(
      `${SUPABASE_URL}/rest/v1/purchases?token=eq.${encodeURIComponent(key)}&product_id=eq.${PRODUCT_ID}&select=id`,
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
    return c.json({ valid: Array.isArray(rows) && rows.length > 0 });
  } catch {
    return c.json({ error: 'License service unavailable' }, 503);
  }
});
