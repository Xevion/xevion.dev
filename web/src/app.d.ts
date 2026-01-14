// See https://svelte.dev/docs/kit/types#app.d.ts

declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}

    interface PageState {
      discordModal?: {
        open: boolean;
        username: string;
      };
    }

    // interface Platform {}
  }
}

export {};
