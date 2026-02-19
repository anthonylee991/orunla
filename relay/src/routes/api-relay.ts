import { Hono } from 'hono';
import { rateLimiter } from '../middleware/rate-limit.js';
import { deviceSockets } from './mcp-relay.js';

/**
 * REST API Relay — proxies HTTP requests to desktop app via WebSocket
 *
 * External services (ChatGPT, n8n, Make.com) send HTTP requests here:
 *   ALL /api/:deviceId/*  →  wrapped in JSON, forwarded via WS, response returned
 *
 * Free for all users. No license key required.
 */

// Pending API responses: requestId -> { resolve, timer }
type PendingEntry = {
  resolve: (value: { status: number; body: any }) => void;
  timer: ReturnType<typeof setTimeout>;
};

export const pendingApiRequests = new Map<string, PendingEntry>();

export const apiRelayRoutes = new Hono();

// Rate limit: 120 req/min per IP (same as MCP relay)
apiRelayRoutes.use('/*', rateLimiter(120));

// Wildcard catch-all: any method, any path under /api/:deviceId/
apiRelayRoutes.all('/:deviceId/*', async (c) => {
  const deviceId = c.req.param('deviceId');
  const fullPath = c.req.path;

  // Strip /api/:deviceId prefix to get the actual API path
  // e.g., /api/abc-123/ingest → /ingest
  const prefixLen = `/api/${deviceId}`.length;
  const path = fullPath.substring(prefixLen) || '/';

  // Check desktop WebSocket connection
  const ws = deviceSockets.get(deviceId);
  if (!ws || ws.readyState !== 1 /* OPEN */) {
    return c.json(
      { error: 'Device not connected. Open the Orunla desktop app.' },
      503
    );
  }

  const requestId = crypto.randomUUID();
  const method = c.req.method;

  // Extract auth headers to forward
  const headers: Record<string, string> = {};
  const forwardHeaders = ['x-api-key', 'authorization', 'content-type'];
  for (const name of forwardHeaders) {
    const val = c.req.header(name);
    if (val) headers[name] = val;
  }

  // Read body for methods that support it
  let body: any = null;
  if (['POST', 'PUT', 'PATCH', 'DELETE'].includes(method)) {
    try {
      const contentType = c.req.header('content-type') || '';
      if (contentType.includes('application/json')) {
        body = await c.req.json();
      } else {
        body = await c.req.text();
      }
    } catch {
      // No body or unreadable — fine
    }
  }

  // Build the relay message
  const message = {
    type: 'api_request',
    id: requestId,
    method,
    path,
    headers,
    body,
  };

  // Send via WebSocket and wait for response (30s timeout)
  return new Promise<Response>((outerResolve) => {
    const timer = setTimeout(() => {
      pendingApiRequests.delete(requestId);
      outerResolve(c.json({ error: 'Request timeout' }, 504));
    }, 30_000);

    pendingApiRequests.set(requestId, {
      resolve: ({ status, body: responseBody }) => {
        clearTimeout(timer);
        pendingApiRequests.delete(requestId);
        outerResolve(c.json(responseBody, status as any));
      },
      timer,
    });

    ws.send(JSON.stringify(message));
  });
});

/**
 * Handle an api_response message from a desktop device.
 * Called from index.ts when we detect msg.type === 'api_response'.
 * Returns true if the response was matched to a pending request.
 */
export function handleApiResponse(msg: {
  id: string;
  status: number;
  body: any;
}): boolean {
  const pending = pendingApiRequests.get(msg.id);
  if (!pending) return false;
  pending.resolve({ status: msg.status, body: msg.body });
  return true;
}
