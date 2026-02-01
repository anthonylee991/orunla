/**
 * Hono environment type for routes that use device auth middleware.
 * The middleware sets deviceId and licenseKey via c.set().
 */
export type AppEnv = {
  Variables: {
    deviceId: string;
    licenseKey: string;
  };
};
