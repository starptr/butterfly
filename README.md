# butterfly

A fast link-shortener with hardcoded optimizations, using Cloudflare Workers.

## Secrets

By design, only 1 user is allowed to write to the KV database. To ensure that
only the authorized user may write, set the `BUTTERFLY_API_TOKEN` environment
variable with the following instructions:

1. Create a file called `secret.json` at the root of the repository.
2. Define a json object and assign a password to the key. This is the token
that you will need to provide to write to the KV database.
3. Run `npx wrangler secret:bulk secret.json` to upload the secret to the worker.

### Local Development

For local development to have access to secrets, you need to define secrets in `.dev.vars` like so:

```
BUTTERFLY_API_TOKEN = "mytoken"
```

This file needs to be manually updated to be kept in sync with `secret.json`.

## API Specification

### `GET /:slug`
Client will receieve status 301 (permanent redirect) to the expanded URL.
Hardcoded redirects also fall under this method, such as those for social media
profiles.

### `POST /add`
This method requires a json body with the following shape:

```
{
	"target": "www.example.com/url-to-shorten",
	"token": "same_token_defined_as_secret"
}
```

and will generate a short url of the url given in `target`. On success, the
response has the following shape:

```
{
	"slug": "slug of short url",
	"url": "full short url"
}
```

## Terraform

Currently, the cloudflare provider for terraform is unable to take multiple files as input. Since building this project generates both a `shim.mjs` and an `index.wasm` file,
terraform cannot deploy it yet.

## Deployment

Using wrangler we can deploy this project easily. See the Usage section below.

---

# Template: worker-rust

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/cloudflare/templates/tree/main/worker-rust)

A template for kick starting a Cloudflare worker project using [`workers-rs`](https://github.com/cloudflare/workers-rs).

This template is designed for compiling Rust to WebAssembly and publishing the resulting worker to Cloudflare's [edge infrastructure](https://www.cloudflare.com/network/).

## Setup

To create a `my-project` directory using this template, run:

```sh
$ npm init cloudflare my-project worker-rust
# or
$ yarn create cloudflare my-project worker-rust
# or
$ pnpm create cloudflare my-project worker-rust
```

> **Note:** Each command invokes [`create-cloudflare`](https://www.npmjs.com/package/create-cloudflare) for project creation.

## Usage

This template starts you off with a `src/lib.rs` file, acting as an entrypoint for requests hitting your Worker. Feel free to add more code in this file, or create Rust modules anywhere else for this project to use.

With `wrangler`, you can build, test, and deploy your Worker with the following commands:

```sh
# compiles your project to WebAssembly and will warn of any issues
$ npm run build

# run your Worker in an ideal development workflow (with a local server, file watcher & more)
$ npm run dev

# deploy your Worker globally to the Cloudflare network (update your wrangler.toml file for configuration)
$ npm run deploy
```

Read the latest `worker` crate documentation here: https://docs.rs/worker

## WebAssembly

`workers-rs` (the Rust SDK for Cloudflare Workers used in this template) is meant to be executed as compiled WebAssembly, and as such so **must** all the code you write and depend upon. All crates and modules used in Rust-based Workers projects have to compile to the `wasm32-unknown-unknown` triple.

Read more about this on the [`workers-rs`](https://github.com/cloudflare/workers-rs) project README.

## Issues

If you have any problems with the `worker` crate, please open an issue on the upstream project issue tracker on the [`workers-rs` repository](https://github.com/cloudflare/workers-rs).
