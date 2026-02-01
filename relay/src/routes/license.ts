import { Hono } from 'hono';
import { rateLimiter } from '../middleware/rate-limit.js';

const SUPABASE_URL = process.env.SUPABASE_URL;
const SUPABASE_SERVICE_KEY = process.env.SUPABASE_SERVICE_KEY;
const PRODUCT_ID = 'orunla_standard';

console.log(`[license] SUPABASE_URL set: ${!!SUPABASE_URL} (${SUPABASE_URL ? SUPABASE_URL.substring(0, 30) + '...' : 'undefined'})`);
console.log(`[license] SUPABASE_SERVICE_KEY set: ${!!SUPABASE_SERVICE_KEY}`);

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
    console.log(`[license] Missing env: URL=${!!SUPABASE_URL} KEY=${!!SUPABASE_SERVICE_KEY}`);
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
      const errText = await res.text().catch(() => '');
      console.log(`[license] Supabase responded ${res.status}: ${errText}`);
      return c.json({ error: 'License service unavailable' }, 503);
    }

    const rows = await res.json();
    console.log(`[license] Supabase returned ${JSON.stringify(rows).substring(0, 200)}`);
    return c.json({ valid: Array.isArray(rows) && rows.length > 0 });
  } catch (err) {
    console.log(`[license] Fetch error: ${err}`);
    return c.json({ error: 'License service unavailable' }, 503);
  }
});
