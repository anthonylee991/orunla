import { db } from '../db/index.js';
import { syncEvents } from '../db/schema.js';
import { lt } from 'drizzle-orm';

const PRUNE_INTERVAL_MS = 6 * 60 * 60 * 1000; // 6 hours
const BUFFER_RETENTION_DAYS = 30;

async function pruneBuffers() {
  const cutoff = new Date(Date.now() - BUFFER_RETENTION_DAYS * 24 * 60 * 60 * 1000);
  try {
    const result = await db
      .delete(syncEvents)
      .where(lt(syncEvents.createdAt, cutoff));
    console.log(`[prune] Deleted sync events older than ${cutoff.toISOString()}`);
  } catch (err) {
    console.error('[prune] Error:', err);
  }
}

export function schedulePrune() {
  // Run immediately on startup
  pruneBuffers();
  // Then every 6 hours
  setInterval(pruneBuffers, PRUNE_INTERVAL_MS);
}
