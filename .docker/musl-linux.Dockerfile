FROM clux/muslrust

RUN apt-get update && apt-get install -y cmake

COPY musl-build.sh /bin/musl-build.sh
RUN chmod +x /bin/musl-build.sh