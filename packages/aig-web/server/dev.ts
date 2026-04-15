import express from "express";
import { createServer as createViteServer } from "vite";
import { registerApiRoutes } from "./api.js";

async function main() {
  const app = express();

  registerApiRoutes(app);

  const vite = await createViteServer({
    server: { middlewareMode: true },
    appType: "spa",
  });
  app.use(vite.middlewares);

  const port = 5180;
  app.listen(port, () => {
    console.log(`\n  aig web: http://localhost:${port}\n`);
  });
}

main();
