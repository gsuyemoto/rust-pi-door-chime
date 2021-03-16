# rust-pi-door-chime
A door chime that will play a custom sound file or text-to-speech when a door is opened (magnetic contact sensor). An early version of this application was a Rust rewrite of the [project](https://learn.adafruit.com/sitcom-sfx-door-trigger/code) posted on Adafruit. Later I decided to change out the PIR with an ultrasonic sensor and used this [project](https://tutorials-raspberrypi.com/raspberry-pi-ultrasonic-sensor-hc-sr04/) as a guide for the wiring and resistors.


This application uses the following Cargo packages:

1. [rust_gpiozero](https://github.com/rahul-thakoor/rust_gpiozero) which allows for an interface to the GPIO pins on the Raspbi
2. [soloud-rs](https://github.com/MoAlyousef/soloud-rs) which allows for an easy way to play .wav files

For the soloud-rs crate, you will need to have Cmake installed and might also need to install libasound2-dev if you don't have the alsa/asoundlib.h around.

The application will check will look in 2 locations for the .wav sound files: the 'media' folder and the root of an attached USB drive. In order for the application to check the USB drive it will need to be mounted. The USB drive can be automatically mounted upon boot if the following is added to the /etc/fstab file:

`/dev/sda1 /mnt/usbname  auto  nosuid,nodev,nofail 0 0`

Where `/dev/sda1` is your USB device and `/mnt/usbname` is the mount directory that you create.

If you have problems playing the sound you might need to change the default sound card by adding this to a ~/.asoundrc file:

`defaults.pcm.!card 1`
`defaults.pcm.!device 0`
