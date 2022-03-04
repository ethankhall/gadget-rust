# Running development server

Some configuration options for how to develop with Gadget

## Caddy + gadget + static website

In this case, you will run Caddy to proxy traffic to gadget. This is a good way to test when you want a full stack including auth.

Here is what you'll need to do. Building that static website is required before running gadget.

1. Run Caddy
2. Build static website
3. Run gadget

There are a few dependencies that are required:
- cargo
- yarn
- npm
- caddy

### Running Caddy

In a shell, run in the repo root.

```
CADDY_WEB_HOST=`hostnamectl --static` caddy -conf sample/Caddyfile
```

This will stay running in the foreground.

### Build static website

In a shell, run in the repo root.

```
cd gadget-ui
yarn
yarn build
```
