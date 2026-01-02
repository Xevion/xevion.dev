# xevion.dev

This is the newest iteration of my personal website.

Instead of focus on playing around or showing off blog posts, this site will focus on presentation,
as a portfolio of what I have learned and what I can do.

## Development

Start the database and dev server:

```bash
pnpm db:start
pnpm dev
```

No `.env` file needed for basic development - sensible defaults are provided. Optional features require environment variables (see `.env.example`).

## Stack

- Hosted by [Vercel][vercel]
- [Next.js][next]
  - [tRPC][trpc]
- [TailwindCSS][tailwind]

[vercel]: https://vercel.com
[next]: https://nextjs.org
[trpc]: https://trpc.io/
[tailwind]: https://tailwindcss.com/
