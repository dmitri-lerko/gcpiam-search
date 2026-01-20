import * as esbuild from 'esbuild';

await esbuild.build({
  entryPoints: ['public/app.ts'],
  bundle: true,
  outfile: 'public/app.js',
  platform: 'browser',
  target: 'es2020',
  sourcemap: true,
});

console.log('✅ Build complete! app.js is ready');
