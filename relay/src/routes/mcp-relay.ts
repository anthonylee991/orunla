import { Hono } from 'hono';
import { streamSSE } from 'hono/streaming';
import { rateLimiter } from '../middleware/rate-limit.js';
import type { WebSocket as WsWebSocket } from 'ws';

/**
 * MCP Relay — bridges browser MCP clients ↔ Desktop app (WebSocket)
 *
 * Supports TWO MCP transports on the same URL:
 *
 * 1. Streamable HTTP (ChatGPT, modern clients):
 *    POST /mcp/:deviceId/sse     → JSON-RPC request/response
 *
 * 2. Legacy SSE (Claude browser):
 *    GET  /mcp/:deviceId/sse     → SSE stream
 *    POST /mcp/:deviceId/message → JSON-RPC messages (with session_id)
 *
 * Desktop app connects via WebSocket:
 *    WS   /mcp/ws?device_id=X   → bidirectional JSON-RPC relay
 *
 * No license key required — MCP relay is free for all users.
 */

// In-memory state: active desktop WebSocket connections
export const deviceSockets = new Map<string, WsWebSocket>();

// In-memory state: active Claude browser SSE sessions per device
// deviceId -> Map<sessionId, write function>
type SseWriter = (event: string, data: string) => void;
const deviceSessions = new Map<string, Map<string, SseWriter>>();

// Pending responses: maps request IDs to session callbacks
// deviceId -> Map<requestId, sessionId>
const pendingRequests = new Map<string, Map<string, string>>();

// Pending Streamable HTTP requests: deviceId -> Map<requestId, resolve function>
// Used for ChatGPT and other clients that use POST-based Streamable HTTP transport
const pendingHttpRequests = new Map<string, Map<string, (data: string) => void>>();

// Session IDs for Streamable HTTP clients (generated on initialize)
const streamableSessionIds = new Map<string, string>();

export const mcpRelayRoutes = new Hono();

// Rate limit all relay routes
mcpRelayRoutes.use('/*', rateLimiter(120));

// CORS for all relay routes (needed for browser-based MCP clients)
mcpRelayRoutes.use('/*', async (c, next) => {
  c.res.headers.set('Access-Control-Allow-Origin', '*');
  c.res.headers.set('Access-Control-Allow-Methods', 'GET, POST, DELETE, OPTIONS');
  c.res.headers.set('Access-Control-Allow-Headers', 'Content-Type, Accept, Mcp-Session-Id');
  c.res.headers.set('Access-Control-Expose-Headers', 'Mcp-Session-Id');
  if (c.req.method === 'OPTIONS') {
    return c.body(null, 204);
  }
  await next();
});

/**
 * POST /mcp/:deviceId/sse
 * Streamable HTTP transport — used by ChatGPT and other modern MCP clients.
 * Receives JSON-RPC POST, forwards to desktop via WebSocket, returns response.
 * Same URL as the SSE endpoint so one URL works for all clients.
 */
mcpRelayRoutes.post('/:deviceId/sse', async (c) => {
  const deviceId = c.req.param('deviceId');

  const ws = deviceSockets.get(deviceId);
  if (!ws || ws.readyState !== 1) {
    return c.json(
      { error: 'Device not connected. Open the Orunla desktop app.' },
      503
    );
  }

  const body = await c.req.json();

  // Requests with an ID need a response (initialize, tools/list, tool calls)
  if (body.id !== undefined && body.id !== null) {
    const requestId = String(body.id);

    const responsePromise = new Promise<string>((resolve, reject) => {
      if (!pendingHttpRequests.has(deviceId)) {
        pendingHttpRequests.set(deviceId, new Map());
      }

      const timeout = setTimeout(() => {
        pendingHttpRequests.get(deviceId)?.delete(requestId);
        reject(new Error('timeout'));
      }, 30000);

      pendingHttpRequests.get(deviceId)!.set(requestId, (data: string) => {
        clearTimeout(timeout);
        resolve(data);
      });
    });

    // Forward to desktop
    ws.send(JSON.stringify(body));

    try {
      const responseData = await responsePromise;

      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
      };

      // Generate session ID on initialize response
      if (body.method === 'initialize') {
        const sessionId = crypto.randomUUID();
        streamableSessionIds.set(deviceId, sessionId);
        headers['Mcp-Session-Id'] = sessionId;
      } else {
        const sid = streamableSessionIds.get(deviceId);
        if (sid) headers['Mcp-Session-Id'] = sid;
      }

      return new Response(responseData, { status: 200, headers });
    } catch {
      return c.json(
        { jsonrpc: '2.0', error: { code: -32000, message: 'Request timeout — desktop app may be unresponsive' }, id: body.id },
        504
      );
    }
  }

  // Notifications (no ID) — fire and forget
  ws.send(JSON.stringify(body));
  return c.body(null, 202);
});

/**
 * DELETE /mcp/:deviceId/sse
 * Streamable HTTP session cleanup. No-op for our relay (sessions are per-request).
 */
mcpRelayRoutes.delete('/:deviceId/sse', async (c) => {
  const deviceId = c.req.param('deviceId');
  streamableSessionIds.delete(deviceId);
  return c.body(null, 200);
});

/**
 * GET /mcp/:deviceId/sse
 * Claude browser connects here to receive SSE events.
 * Sends an "endpoint" event telling Claude where to POST messages.
 */
mcpRelayRoutes.get('/:deviceId/sse', async (c) => {
  const deviceId = c.req.param('deviceId');

  // Check if the desktop device is connected
  const ws = deviceSockets.get(deviceId);
  if (!ws || ws.readyState !== 1 /* OPEN */) {
    return c.json(
      { error: 'Device not connected. Open the Orunla desktop app.' },
      503
    );
  }

  const sessionId = crypto.randomUUID();

  return streamSSE(c, async (stream) => {
    // Register this SSE session
    if (!deviceSessions.has(deviceId)) {
      deviceSessions.set(deviceId, new Map());
    }

    const writer: SseWriter = (event: string, data: string) => {
      stream.writeSSE({ event, data }).catch(() => {
        // Stream closed — will be cleaned up below
      });
    };

    deviceSessions.get(deviceId)!.set(sessionId, writer);

    // Send the endpoint event (tells Claude where to POST messages)
    // Must include /mcp prefix since routes are mounted at app.route('/mcp', ...)
    // Use forwarded headers to get the public URL (behind Railway's reverse proxy,
    // c.req.url gives the internal localhost address, not the public domain)
    const proto = c.req.header('x-forwarded-proto') || 'https';
    const host = c.req.header('x-forwarded-host') || c.req.header('host') || 'localhost';
    const endpointUrl = `${proto}://${host}/mcp/${deviceId}/message?session_id=${sessionId}`;
    await stream.writeSSE({ event: 'endpoint', data: endpointUrl });

    // Keep alive with periodic pings
    const keepAlive = setInterval(() => {
      stream.writeSSE({ event: 'ping', data: '' }).catch(() => {
        clearInterval(keepAlive);
      });
    }, 15000);

    // Wait until the stream closes (client disconnects)
    stream.onAbort(() => {
      clearInterval(keepAlive);
      deviceSessions.get(deviceId)?.delete(sessionId);
      if (deviceSessions.get(deviceId)?.size === 0) {
        deviceSessions.delete(deviceId);
      }
      // Clean up pending requests for this session
      const pending = pendingRequests.get(deviceId);
      if (pending) {
        for (const [reqId, sid] of pending.entries()) {
          if (sid === sessionId) {
            pending.delete(reqId);
          }
        }
      }
    });

    // Keep the stream open indefinitely (until abort)
    await new Promise(() => {}); // Never resolves — stream stays open
  });
});

/**
 * POST /mcp/:deviceId/message?session_id=X
 * Claude browser sends JSON-RPC messages here.
 * The relay forwards them to the desktop via WebSocket.
 */
mcpRelayRoutes.post('/:deviceId/message', async (c) => {
  const deviceId = c.req.param('deviceId');
  const sessionId = c.req.query('session_id');

  if (!sessionId) {
    return c.json({ error: 'Missing session_id query parameter' }, 400);
  }

  // Verify the SSE session exists
  const sessions = deviceSessions.get(deviceId);
  if (!sessions || !sessions.has(sessionId)) {
    return c.json({ error: 'Session not found' }, 404);
  }

  // Check desktop WebSocket connection
  const ws = deviceSockets.get(deviceId);
  if (!ws || ws.readyState !== 1) {
    return c.json({ error: 'Device disconnected' }, 503);
  }

  const body = await c.req.json();

  // Track which session this request came from (so we can route the response)
  if (body.id !== undefined && body.id !== null) {
    if (!pendingRequests.has(deviceId)) {
      pendingRequests.set(deviceId, new Map());
    }
    pendingRequests.get(deviceId)!.set(String(body.id), sessionId);
  }

  // Forward the JSON-RPC message to the desktop app via WebSocket
  ws.send(JSON.stringify(body));

  return c.body(null, 202);
});

/**
 * Handle incoming WebSocket messages from a desktop app.
 * Called by the WebSocket server handler in index.ts.
 */
export function handleDeviceMessage(deviceId: string, data: string): void {
  let msg: any;
  try {
    msg = JSON.parse(data);
  } catch {
    console.error(`[mcp-relay] Invalid JSON from device ${deviceId}`);
    return;
  }

  // Route the response to the correct client
  const requestId = msg.id !== undefined ? String(msg.id) : null;

  // Check Streamable HTTP pending requests first (ChatGPT, etc.)
  if (requestId) {
    const httpPending = pendingHttpRequests.get(deviceId);
    if (httpPending?.has(requestId)) {
      const resolve = httpPending.get(requestId)!;
      httpPending.delete(requestId);
      resolve(data);
      return;
    }
  }

  // Fall through to SSE routing (Claude browser, etc.)
  const pending = pendingRequests.get(deviceId);
  const sessions = deviceSessions.get(deviceId);

  if (!sessions) return;

  if (requestId && pending?.has(requestId)) {
    // We know exactly which session this response belongs to
    const sessionId = pending.get(requestId)!;
    pending.delete(requestId);

    const writer = sessions.get(sessionId);
    if (writer) {
      writer('message', data);
    }
  } else {
    // Broadcast to all sessions for this device (notifications, etc.)
    for (const writer of sessions.values()) {
      writer('message', data);
    }
  }
}

/**
 * Clean up when a desktop device disconnects.
 */
export function handleDeviceDisconnect(deviceId: string): void {
  deviceSockets.delete(deviceId);
  pendingRequests.delete(deviceId);
  pendingHttpRequests.delete(deviceId);
  streamableSessionIds.delete(deviceId);
  // Note: we don't close SSE sessions here — they'll get 503 on next message
  // and Claude will reconnect when the device comes back
  console.log(`[mcp-relay] Device ${deviceId} disconnected`);
}
