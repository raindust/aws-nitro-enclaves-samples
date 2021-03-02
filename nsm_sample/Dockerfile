# start the Docker image from the Alpine Linux distribution
FROM alpine:latest
# copy the vsock-sample binary to the Docker file
COPY target/x86_64-unknown-linux-musl/release/nsm_sample .
# start the server application inside the enclave
CMD ./nsm_sample server --port 5005