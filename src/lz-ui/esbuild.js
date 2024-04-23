// ESbuild pipeline, invoked as a trunk pre_build stage hook:
const esbuild = require("esbuild");

esbuild
  .build({
    entryPoints: ["js/package.js"],
    bundle: true,
    outfile: "./package.js",
    format: "esm",
    minify: true,
    define: {
      SENTRY_DSN: `"${process.env.SENTRY_DSN}"`,
    },
  })
  .catch(() => process.exit(1));
