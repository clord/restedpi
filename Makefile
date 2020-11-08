
restedpi:
	docker rm -f restedpi || true
	docker build -t build-restedpi-image .
	docker create --rm -ti --name restedpi build-restedpi-image bash
	docker cp restedpi:/root/restedpi/target/arm-unknown-linux-gnueabihf/release/restedpi restedpi

