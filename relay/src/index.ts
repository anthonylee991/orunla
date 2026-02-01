import { Hono } from 'hono';
import { serve } from '@hono/node-server';
import { licenseRoutes } from './routes/license.js';
import { registerRoutes } from './routes/register.js';
import { syncRoutes } from './routes/sync.js';
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

// Start pruning job
schedulePrune();

const PORT = parseInt(process.env.PORT || '3000', 10);

console.log(`Orunla relay starting on port ${PORT}`);
serve({ fetch: app.fetch, port: PORT });
