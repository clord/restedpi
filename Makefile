image:
	nix build .#rpi4

build/restedpi:
	earthly +build

deploy: build/restedpi
	scp build/restedpi  raspberrypi.z.odago.ca:~/new-restedpi
	ssh raspberrypi.z.odago.ca "sudo mv ~/new-restedpi /usr/local/bin/restedpi;  sudo systemctl restart restedpi"
