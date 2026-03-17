FROM golang:1.24 as builder

WORKDIR /usr/src/app

# Copy proto
COPY ./proto ./proto

# Copy gateway
COPY ./gateway ./gateway
WORKDIR /usr/src/app/gateway

RUN go mod download
# Go build
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o gateway main.go

# Final minimal image
FROM alpine:3.19
RUN apk --no-cache add ca-certificates
COPY --from=builder /usr/src/app/gateway/gateway /root/gateway

EXPOSE 8080
CMD ["/root/gateway"]
