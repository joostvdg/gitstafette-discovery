
get-servers-local:
	@echo "Getting servers from local"
	cargo run --bin client -- get-servers

new-server-local:
	@echo "Creating new server on local"
	cargo run --bin client -- register-server --id "001" --name "local" --host "localhost" --port "50051" --repositories "123456,456678" --version "0.1.0" 

get-hubs-local:
	@echo "Getting hubs from local"
	cargo run --bin client -- get-hubs

new-hub-local:
	@echo "Creating new hub on local"
	cargo run --bin client -- register-hub --id "001" --name "local" --host "localhost" --port "50051" --repositories "123456,456678" --relay-host "N/A" --relay-port "N/A" --version "0.1.0" 