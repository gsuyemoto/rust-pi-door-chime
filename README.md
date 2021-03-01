# rust-pi-door-chime
A door chime that will play a custom sound file or text-to-speech when a door is opened (magnetic contact sensor).


This application uses the following Cargo packages:

1. [rust_gpiozero](https://github.com/rahul-thakoor/rust_gpiozero) which allows for an interface to the GPIO pins on the Raspbi
2. [soloud-rs](https://github.com/MoAlyousef/soloud-rs) which allows for an easy way to play .wav files

The application will check will look in 2 locations for the .wav sound files: the 'media' folder and the root of an attached USB drive. In order for the application to check the USB drive it will need to be mounted. The USB drive can be automatically mounted upon boot if the following is added to the /etc/fstab file:

`/dev/sda1 /mnt/usbname  auto  nosuid,nodev,nofail 0 0`

Where `/dev/sda1` is your USB device and `/mnt/usbname` is the mount directory that you create.
