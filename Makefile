DEPLOY := '10.68.2.7'

build-pi: 
	cargo build --target=armv7-unknown-linux-gnueabihf 

deploy-pi: build-pi
	scp target/armv7-unknown-linux-gnueabihf/debug/restedpi-rust pi@${DEPLOY}:~/ 
       
test-deploy-pi: deploy-pi
	ssh pi@${DEPLOY}  "~/rustedpi-rust"


