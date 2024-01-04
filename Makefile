run:
	@echo "Running local"
	cargo run --bin server -- --listener-address 127.0.0.1

client:
	@echo "Running client"
	cargo run --bin client -- --hostname 127.0.0.1 info-registration-loop --info-host 127.0.0.1 --info-port 50051 --info-protocol http

get-servers-local:
	@echo "Getting servers from local"
	cargo run --bin client -- --hostname 127.0.0.1 get-servers

new-server-local:
	@echo "Creating new server on local"
	cargo run --bin client -- --hostname 127.0.0.1 register-server --id "001" --name "local" --host "localhost" --port "50051" --repositories "123456,456678" --version "0.1.0"

get-hubs-local:
	@echo "Getting hubs from local"
	cargo run --bin client -- --hostname 127.0.0.1 get-hubs

new-hub-local:
	@echo "Creating new hub on local"
	cargo run --bin client -- --hostname 127.0.0.1 register-hub --id "001" --name "local" --host "localhost" --port "50051" --repositories "123456,456678" --relay-host "N/A" --relay-port "N/A" --version "0.1.0"

.PHONY: dpush-alpine
dpush-alpine:
	docker buildx build . \
		-f docker/alpine/Dockerfile \
		--platform linux/amd64,linux/arm64 \
		--tag ghcr.io/joostvdg/gitstafette-discovery:$(VERSION)-alpine \
		--build-arg BUILDKIT_INLINE_BUILDINFO_ATTRS=1 \
		--provenance=false --sbom=false --push

.PHONY: dpush-alpine-amd
dpush-alpine-amd:
	docker buildx build . -f docker/alpine/Dockerfile --platform linux/amd64 --tag ghcr.io/joostvdg/gitstafette-discovery:$(VERSION)-alpine --provenance=false --sbom=false --push

.PHONY: dpush-amd
dpush-amd:
	docker buildx build . --file docker/debian/Dockerfile --platform linux/amd64 --tag ghcr.io/joostvdg/gitstafette-discovery:$(VERSION)-amd --provenance=false --sbom=false --push

.PHONY: dpush-arm
dpush-arm:
	docker buildx build . --file docker/debian/Dockerfile --platform linux/arm64 --tag ghcr.io/joostvdg/gitstafette-discovery:$(VERSION)-arm --provenance=false --sbom=false --push

.PHONY: dpush
dpush:
	@echo "Building and pushing Container image to ghcr.io/joostvdg/gitstafette-discover:${VERSION}}"
	docker buildx build . \
		--file docker/debian/Dockerfile \
		--platform linux/amd64,linux/arm64 \
		--tag ghcr.io/joostvdg/gitstafette-discovery:$(VERSION)-debian \
		--build-arg BUILDKIT_INLINE_BUILDINFO_ATTRS=1 \
		--provenance=false --sbom=false --push


.PHONY:
drun-alpine:
	docker run --rm -it -p 50051:50051 ghcr.io/joostvdg/gitstafette-discovery:$(VERSION)-alpine