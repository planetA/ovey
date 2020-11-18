echo "execute this from the ovey userland CLI directory"

cargo build --all

cargo run --bin ovey_cli -- new \
  --vnetid c929e96d-6285-4528-b98e-b364d64790ae \
  --guid dead:beef:0bad:f00d \
  --name ovey0 \
  --parent rxe0


