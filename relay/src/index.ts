import { Hono } from 'hono';
import { serve } from '@hono/node-server';
import { WebSocketServer } from 'ws';
import { licenseRoutes } from './routes/license.js';
import { registerRoutes } from './routes/register.js';
import { syncRoutes } from './routes/sync.js';
import {
  mcpRelayRoutes,
  deviceSockets,
  handleDeviceMessage,
  handleDeviceDisconnect,
} from './routes/mcp-relay.js';
import { schedulePrune } from './jobs/prune.js';

const app = new Hono();

// Health check
app.get('/health', (c) => c.json({ status: 'ok', service: 'orunla-relay' }));

// License validation (no auth required -- user doesn't have an API key yet)
app.route('/v1/license', licenseRoutes);

// Device registration (validates license key against Supabase)
app.route('/orunla/register', registerRoutes);

// Sync endpoints (requires device auth)
app.route('/orunla', syncRoutes);

// MCP relay (SSE + message proxy — free for all users)
app.route('/mcp', mcpRelayRoutes);

// Start pruning job
schedulePrune();

const PORT = parseInt(process.env.PORT || '3000', 10);

console.log(`Orunla relay starting on port ${PORT}`);
const server = serve({ fetch: app.fetch, port: PORT });

// WebSocket server for desktop app connections
// Desktop apps connect to ws://relay/mcp/ws?device_id=X
const wss = new WebSocketServer({ noServer: true });

server.on('upgrade', (request, socket, head) => {
  const url = new URL(request.url || '', `http://localhost:${PORT}`);

  if (url.pathname === '/mcp/ws') {
    const deviceId = url.searchParams.get('device_id');

    if (!deviceId) {
      socket.write('HTTP/1.1 400 Bad Request\r\n\r\n');
      socket.destroy();
      return;
    }

    wss.handleUpgrade(request, socket, head, (ws) => {
      console.log(`[mcp-relay] Device ${deviceId} connected via WebSocket`);

      // Close any existing connection for this device (reconnect scenario)
      const existing = deviceSockets.get(deviceId);
      if (existing) {
        existing.close(1000, 'Replaced by new connection');
      }

      // Register the WebSocket
      deviceSockets.set(deviceId, ws);

      ws.on('message', (data) => {
        handleDeviceMessage(deviceId, data.toString());
      });

      ws.on('close', () => {
        // Only disconnect if this socket is still the active one for this device.
        // Prevents a replaced (old) connection's close handler from removing the new one.
        if (deviceSockets.get(deviceId) === ws) {
          handleDeviceDisconnect(deviceId);
        }
      });

      ws.on('error', (err) => {
        console.error(`[mcp-relay] WebSocket error for ${deviceId}:`, err.message);
        if (deviceSockets.get(deviceId) === ws) {
          handleDeviceDisconnect(deviceId);
        }
      });

      // Send a welcome message so the client knows it's connected
      ws.send(JSON.stringify({ type: 'connected', device_id: deviceId }));
    });
  } else {
    // Not a known WebSocket path — reject
    socket.write('HTTP/1.1 404 Not Found\r\n\r\n');
    socket.destroy();
  }
});
