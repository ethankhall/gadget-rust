# Gadget

Gadget is a link shortener.

So you can do something like `gto.cx/test` and it will take you to `google.com`.

There is an expression language that can be used for things like `gto.cx/google this is a long query`

## Private Deployment

These steps are how to run Gadget inside a house, where there will
be no public internet access.

While gadget could handle public internet access, you'll want things
like TLS and auth when trying to access the configuration pages.

### Systemd Unit File

This systemd file will keep the service up and running using docker.

```
# /etc/systemd/system/gadget-service.service
[Unit]
Description=Gadget Service Container
After=docker.service
Requires=docker.service

[Service]
TimeoutStartSec=0
Restart=always
ExecStartPre=-/usr/bin/docker stop gadget-service
ExecStartPre=-/usr/bin/docker rm gadget-service
ExecStart=/usr/bin/docker run \
    --rm \
    --name gadget-service \
    -v /mnt/<path to config dir>/gadget/:/opt/gadget/ \
    -p 8080:8080 \
    docker.pkg.github.com/ethankhall/gadget-rust/gadget:latest \
    /app/bin/gadget --database-url file:///opt/gadget/config.json


[Install]
WantedBy=multi-user.target
```

## Deploy to the Public Internet

Follow the same steps for a private deployment and add the caddy configuration.

### Caddy configuration

Caddy has some excellent [documentation](https://github.com/caddyserver/caddy/tree/master/dist/init/linux-systemd) for how to set it up.

The following file will give a reasonabaly secure public internet
deployment. This is a simple config, and could be made way more 
complicated if a more complicated auth was used.

```text
#/etc/caddy/Caddyfile

example.com { 
        proxy / localhost:8080 {
            transparent
        }

        jwt {
            path /_gadget
            redirect /login?backTo={rewrite_uri}
            allow sub bob
        }

        login {
            simple bob=password
        }
}
```