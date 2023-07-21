#!/usr/bin/env deno run --allow-read=chatServer.db,chatServer.db-journal --allow-write=chatServer.db,chatServer.db-journal --allow-net

import { z } from "https://deno.land/x/zod@v3.21.4/mod.ts";
import * as sqlite from "https://deno.land/x/sqlite@v3.7.2/mod.ts";
import * as oak from "https://deno.land/x/oak@v12.6.0/mod.ts";

const port = 8080;

const db = new sqlite.DB("chatServer.db");

db.execute(`
  CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message TEXT
  )
`);

const router = new oak.Router();

router.post("/post", async (ctx) => {
  const body = await ctx.request.body({ type: "json" }).value;

  if (typeof body !== "string") {
    ctx.response.body = "Bad request";
    ctx.response.status = 400;
    return;
  }

  db.query(
    "INSERT INTO messages (message) VALUES (:message)",
    { message: body },
  );
});

router.get("/recent", async (ctx) => {
  const results = z.array(z.tuple([z.string()])).parse(
    db.query("SELECT message FROM messages ORDER BY id DESC LIMIT 10"),
  );

  const messages = results.map(([message]) => message);
  messages.reverse();

  ctx.response.body = messages;
});

const app = new oak.Application();

app.use(router.routes());
app.use(router.allowedMethods());

app.addEventListener("listen", () => {
  console.log(`Listening on port ${port}`);
});

await app.listen({ port });
