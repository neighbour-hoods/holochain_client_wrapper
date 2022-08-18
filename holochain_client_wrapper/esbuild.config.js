require('esbuild').build({
  entryPoints: ['./submodules/holochain-client-js/src/index.ts'],
  bundle: true,
  outfile: 'crates/holochain_client_wrapper/src/holochain_client_wrapper.js',
  format: 'esm',
}).then(_ => console.log("success 🚀"))
  .catch(() => process.exit(1))
