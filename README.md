# Christmas Rattlesnake

First and foremost this is meant to be a fun project that might inspire folks to embedded development.

This project was originally started as a demonstration for my Engineering Merit Badge class I would
give to the scouts.  It started as a C/C++ codebase targetting an Arduino Atmel ATMega 8 bit processor.
With the release of the ESP32-C3-DevKitM-1 I felt like it was time to update it and modernize it with
Rust.

Since folks are aways asking me where they could get the source code I am publishing it here.

## What it does

The idea is that as people walk into the tent and walk up to the front table that the motion would be
detected and as the person gets closer it would cause a string of christmas lights to strobe at an
increasing frequency.

It uses an IR sensor to detect motion and then an ultrasonic sensor to detect distance.
The LCD screen will show the detected distance and the trigger the solid state relay to actuate the
christmas tree light strand.

With the transition to the ESP32 dev board.  There is an RGB Led that I cycle through a rainbow of
colors.

