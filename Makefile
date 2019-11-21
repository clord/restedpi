DEPLOY := '10.68.2.7'

restedpi-rust:
	docker rm -f restedpi || true
	docker build -t build-restedpi-image .
	docker create --rm -ti --name restedpi build-restedpi-image bash
	docker cp restedpi:/root/restedpi-rust/target/armv7-unknown-linux-gnueabihf/release/restedpi-rust restedpi-rust

deploy-binary: restedpi-rust
	scp restedpi-rust pi@${DEPLOY}:~/restedpi-rust

test-binary: deploy-binary
	ssh pi@${DEPLOY}  "~/rustedpi-rust"

