FROM abiosoft/caddy:1.0.1

COPY Caddyfile /etc/Caddyfile
ENV ACME_AGREE=true