FROM fedora:33
WORKDIR /usr/local/bin
COPY ./target/release/sku_image_microservice /usr/local/bin/sku_image_microservice
STOPSIGNAL SIGINT
ENTRYPOINT ["sku_image_microservice"]
