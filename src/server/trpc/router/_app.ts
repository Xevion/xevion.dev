import { exampleRouter } from "@/server/trpc/router/example";
import { router } from "@/server/trpc/trpc";

export const appRouter = router({
  example: exampleRouter,
});

// export type definition of API
export type AppRouter = typeof appRouter;
