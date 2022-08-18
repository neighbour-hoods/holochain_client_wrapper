# `holochain_client_wrapper`

this is an experimental repo which uses `wasm-bindgen` to construct a Rust FFI wrapper/shim for the officially-supported-by-Holochain [holochain-client-js](https://github.com/holochain/holochain-client-js) library.

it is quite prototype-y at this stage, and its feature-completeness is largely driven by NH concerns around specific widgets/hApps we want to build. it should not be considered solid, and maintainership is dependent on NH engineering priorities.

## how to use this repo

from a separate repo, in which you wish to use this JS/Rust shim:

```
# add as submodule
git submodule add git@github.com:neighbour-hoods/holochain_client_wrapper.git crates/holochain_client_wrapper

# recursively update submodules
git submodule update --init --recursive

# install `esbuild`
npm install esbuild

# generate JS bundle from `holochain-client-js`. `holochain_client_wrapper` expects to see
# this so that it can wasm-bindgen-it.
./node_modules/.bin/esbuild ./crates/holochain_client_wrapper/submodules/holochain-client-js/src/index.ts --format=esm --bundle --outfile=./crates/holochain_client_wrapper/holochain_client_wrapper/src/holochain_client_wrapper.js
```

then, add other crates in `crates/` which depend on `crates/holochain_client_wrapper` as a normal Rust/wasm crate.

## disclaimer about risks inherent in use of this repo

this repo is a relatively thin wrapper for `holochain-client-js`. as such, if it is to remain "faithful to Holochain", it will have to change to match that repo.

this repo is trying to provide a Rust-y interface to Holochain, but because Javascript was chosen as the language for the official Holochain web client API, certain risks are unavoidable. these include all manner of runtime errors, and de/serialization errors which occur at the Rust-Javascript boundary. furthermore, use of this library adds complexity & bug-surface-area to one's hApp UI over building with Javascript and using `holochain-client-js`.

still, I @mhueschen believe Rust is a promising tool for web/wasm frontends, and believe the benefits outweigh the costs of Rust/Javascript interoperation.

### slight tangent about future Holochain web interfaces

further, I believe that it could be possible to someday rewrite [`holochain_websocket`](https://github.com/holochain/holochain/tree/develop/crates/holochain_websocket) to abstract over the choice of underlying websocket, enabling the package to target both non-WASM Rust (as it currently does) and WASM Rust.

this would allow Holochain to use its singular de/serialization interface [`SerializedBytes`](https://docs.rs/hdk/latest/hdk/prelude/struct.SerializedBytes.html) for transit of values between frontend and backend, keeping all of that process within Rust.

in my opinion, this would have immense benefits for code safety and maintenance of the total "Holochain system". instead of having to manually "check for agreement" between the Rust "backend" de/serialization + websocket code, and the Javascript "frontend" de/serialization + websocket code, the process could be automated with Rust's typechecker.

if that were to be done, the `holochain-client-js` JS interface could be swapped for a [`wasm-bindgen` generated JS module](https://rustwasm.github.io/wasm-bindgen/contributing/design/exporting-rust.html#exporting-a-function-to-js) which could expose a similar or identical API structure to what is currently provided.

#### potential concerns about this approach

it's possible that JS/Wasm call overhead is substantial enough that switching the core web-interface-client implementation to Rust/wasm would incur substantial overhead for JS web frontends. my relatively-uninformed suspicion is that the network latency on communicating with the Holochain conductor would outweigh this, though.

it is possible that Typescript support would suffer from a generated-by-`wasm-bindgen` JS client library. it is possible that this would go against Holochain's interest in providing solid-ish Typescript types for frontend developers to bolster their hApp frontends with.

#### tl;dr

it's possible that Holochain could rewrite their "web client" library in Rust, and then generate a Javascript shim from that. this would be essentially the reverse of the process I have done here.

if you are interested in such an approach, please open or comment on an issue on this repo to indicate that. I currently don't know of other devs interested in Rust web frontends for Holochain, and it would help to gauge interest.
