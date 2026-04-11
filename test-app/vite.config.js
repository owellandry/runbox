import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  server: {
    fs: {
      allow: [
        process.cwd(),
        path.resolve(process.cwd(), '../pkg'),
      ],
    },
  },
});
