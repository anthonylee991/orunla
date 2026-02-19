import { Hono } from 'hono';
import { streamSSE } from 'hono/streaming';
import { rateLimiter } from '../middleware/rate-limit.js';
import type { WebSocket as WsWebSocket } from 'ws';

/**
 * MCP Relay — bridges Claude browser (SSE) ↔ Desktop app (WebSocket)
 *
 * Claude browser connects via standard MCP SSE protocol:
 *   GET  /mcp/:deviceId/sse     → SSE stream
 *   POST /mcp/:deviceId/message → JSON-RPC messages
 *
 * Desktop app connects via WebSocket:
 *   WS   /mcp/ws?device_id=X   → bidirectional JSON-RPC relay
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

export const mcpRelayRoutes = new Hono();

// Rate limit all relay routes
mcpRelayRoutes.use('/*', rateLimiter(120));

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
    // Use the request's origin to build an absolute URL (Claude's connector may not resolve relative URLs)
    const requestUrl = new URL(c.req.url);
    const endpointUrl = `${requestUrl.origin}/mcp/${deviceId}/message?session_id=${sessionId}`;
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

  // Route the response to the correct SSE session
  const requestId = msg.id !== undefined ? String(msg.id) : null;
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
  // Note: we don't close SSE sessions here — they'll get 503 on next message
  // and Claude will reconnect when the device comes back
  console.log(`[mcp-relay] Device ${deviceId} disconnected`);
}
