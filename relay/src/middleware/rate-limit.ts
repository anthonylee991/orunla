import type { Context, Next, MiddlewareHandler } from 'hono';

const hitCounts = new Map<string, { count: number; resetAt: number }>();

/**
 * Simple in-memory rate limiter per IP.
 * @param maxPerMinute Maximum requests per minute per IP.
 */
export function rateLimiter(maxPerMinute: number): MiddlewareHandler {
  return async (c: Context, next: Next) => {
    const ip =
      c.req.header('x-forwarded-for')?.split(',')[0]?.trim() ||
      c.req.header('x-real-ip') ||
      'unknown';

    const now = Date.now();
    const entry = hitCounts.get(ip);

    if (!entry || now > entry.resetAt) {
      hitCounts.set(ip, { count: 1, resetAt: now + 60_000 });
    } else {
      entry.count++;
      if (entry.count > maxPerMinute) {
        return c.json({ error: 'Rate limit exceeded' }, 429);
      }
    }

    await next();
  };
}
