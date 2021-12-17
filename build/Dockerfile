FROM alpine:3
RUN apk add curl

ARG hapi_server_base_url
ENV hapi_server_base_url=$hapi_server_base_url
WORKDIR /hapi-server-initializer

COPY ./compose/hapi-server-initializer/* ./

ENTRYPOINT ["/hapi-server-initializer/init-infektionsmedicinsk.sh"]
