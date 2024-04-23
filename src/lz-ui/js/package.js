import * as Sentry from "@sentry/browser";
import { wasmIntegration } from "@sentry/wasm";

export function setup_sentry() {
  Sentry.init({
    dsn: SENTRY_DSN, // SENTRY_DSN is `define`d in esbuild.js, from $SENTRY_DSN
    integrations: [wasmIntegration()],
  });
}
