DEPLOY := '10.68.2.7'

restedpi:
	docker rm -f restedpi || true
	docker build -t build-restedpi-image .
	docker create --rm -ti --name restedpi build-restedpi-image bash
	docker cp restedpi:/root/restedpi/target/armv7-unknown-linux-gnueabihf/release/restedpi restedpi

deploy-binary: restedpi
	scp restedpi pi@${DEPLOY}:~/restedpi

test-binary: deploy-binary
	ssh pi@${DEPLOY}  "~/restedpi"

